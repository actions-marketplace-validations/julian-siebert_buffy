use crate::{
    configs::profiles::typescript::TypeScript,
    dependencies::{node, npm, protoc, protoc_gen_ts_proto, tsc},
    error::Result,
    targets::{
        context::Context,
        typescript::{
            git::{build_typescript_profile_git_target, publish_typescript_profile_git_target},
            npm::{build_typescript_profile_npm_target, publish_typescript_profile_npm_target},
        },
    },
};

pub mod git;
mod helpers;
pub mod npm;

pub async fn check_typescript_profile_target(ctx: Context, _ts: &TypeScript) -> Result<()> {
    ctx.pb.set_message("Checking protoc...");
    protoc()?;

    ctx.pb.set_message("Checking protoc-gen-ts_proto...");
    protoc_gen_ts_proto()?;

    ctx.pb.set_message("Checking node...");
    node()?;

    ctx.pb.set_message("Checking npm...");
    npm()?;

    ctx.pb.set_message("Checking tsc...");
    tsc()?;

    if let TypeScript::Git(_) = _ts {
        ctx.pb.set_message("Checking git...");
        crate::dependencies::git()?;
    }

    ctx.finish_check();

    Ok(())
}

pub async fn build_typescript_profile_target(ctx: Context, ts: &TypeScript) -> Result<()> {
    check_typescript_profile_target(ctx.clone(), ts).await?;

    helpers::generate_ts_code(&ctx, ts).await?;
    helpers::generate_index_ts(&ctx).await?;

    match ts {
        TypeScript::Npm(n) => build_typescript_profile_npm_target(ctx.clone(), n).await?,
        TypeScript::Git(g) => build_typescript_profile_git_target(ctx.clone(), g).await?,
    }

    ctx.finish_build();

    Ok(())
}

pub async fn publish_typescript_profile_target(ctx: Context, ts: &TypeScript) -> Result<()> {
    match ts {
        TypeScript::Npm(n) => publish_typescript_profile_npm_target(ctx.clone(), n).await?,
        TypeScript::Git(g) => publish_typescript_profile_git_target(ctx.clone(), g).await?,
    }
    Ok(())
}
