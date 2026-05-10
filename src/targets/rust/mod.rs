use tokio::process::Command;

use crate::{
    configs::profiles::rust::Rust,
    dependencies::{cargo, protoc, protoc_gen_prost, protoc_gen_prost_crate, protoc_gen_tonic},
    error::Result,
    targets::{
        context::Context,
        rust::{
            crates::{build_rust_profile_crate_target, publish_rust_profile_crate_target},
            git::{build_rust_profile_git_target, publish_rust_profile_git_target},
        },
    },
};

mod codegen;
pub mod crates;
pub mod git;
pub mod helpers;

pub async fn check_rust_profile_target(ctx: Context, rust: &Rust) -> Result<()> {
    ctx.pb.set_message("Checking protoc...");
    protoc()?;

    ctx.pb.set_message("Checking protoc-gen-prost...");
    protoc_gen_prost()?;

    ctx.pb.set_message("Checking protoc-gen-prost-crate...");
    protoc_gen_prost_crate()?;

    ctx.pb.set_message("Checking cargo...");
    cargo()?;

    let grpc = match rust {
        Rust::Crate(c) => c.grpc,
        Rust::Git(g) => g.grpc,
    };
    if grpc {
        ctx.pb.set_message("Checking protoc-gen-tonic...");
        protoc_gen_tonic()?;
    }

    // variant-specific checks
    match rust {
        Rust::Crate(_) => {} // no extra deps
        Rust::Git(_) => {
            ctx.pb.set_message("Checking git...");
            crate::dependencies::git()?;
        }
    }

    ctx.finish_check();

    Ok(())
}

pub async fn build_rust_profile_target(ctx: Context, rust: &Rust) -> Result<()> {
    // shared codegen step
    codegen::generate_rust_code(&ctx, rust).await?;

    // shared lib.rs flattening
    codegen::generate_lib_rs(&ctx).await?;

    // variant-specific Cargo.toml + build
    match rust {
        Rust::Crate(c) => build_rust_profile_crate_target(ctx.clone(), c).await?,
        Rust::Git(g) => build_rust_profile_git_target(ctx.clone(), g).await?,
    }

    // verify
    ctx.pb.set_message("Verifying cargo build...");
    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--quiet"]).current_dir(&ctx.target_path);
    ctx.run(&mut cmd).await?;

    ctx.finish_build();

    Ok(())
}

pub async fn publish_rust_profile_target(ctx: Context, rust: &Rust) -> Result<()> {
    match rust {
        Rust::Crate(c) => publish_rust_profile_crate_target(ctx.clone(), c).await?,
        Rust::Git(g) => publish_rust_profile_git_target(ctx.clone(), g).await?,
    }
    Ok(())
}
