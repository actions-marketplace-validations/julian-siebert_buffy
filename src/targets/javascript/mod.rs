use crate::{
    configs::profiles::javascript::JavaScript,
    dependencies::{node, npm, protoc, protoc_gen_grpc_web, protoc_gen_js},
    error::Result,
    targets::{
        context::Context,
        javascript::{
            git::{build_javascript_profile_git_target, publish_javascript_profile_git_target},
            npm::{build_javascript_profile_npm_target, publish_javascript_profile_npm_target},
        },
    },
};

pub mod git;
mod helpers;
pub mod npm;

pub async fn check_javascript_profile_target(ctx: Context, js: &JavaScript) -> Result<()> {
    ctx.pb.set_message("Checking protoc...");
    protoc()?;

    ctx.pb.set_message("Checking protoc-gen-js...");
    protoc_gen_js()?;

    ctx.pb.set_message("Checking node...");
    node()?;

    ctx.pb.set_message("Checking npm...");
    npm()?;

    let grpc = match js {
        JavaScript::Npm(n) => n.grpc,
        JavaScript::Git(g) => g.grpc,
    };
    if grpc {
        ctx.pb.set_message("Checking protoc-gen-grpc-web...");
        protoc_gen_grpc_web()?;
    }

    if let JavaScript::Git(_) = js {
        ctx.pb.set_message("Checking git...");
        crate::dependencies::git()?;
    }

    ctx.finish_check();

    Ok(())
}

pub async fn build_javascript_profile_target(ctx: Context, js: &JavaScript) -> Result<()> {
    check_javascript_profile_target(ctx.clone(), js).await?;
    helpers::generate_js_code(&ctx, js).await?;

    match js {
        JavaScript::Npm(n) => build_javascript_profile_npm_target(ctx.clone(), n).await?,
        JavaScript::Git(g) => build_javascript_profile_git_target(ctx.clone(), g).await?,
    }

    ctx.finish_build();

    Ok(())
}

pub async fn publish_javascript_profile_target(ctx: Context, js: &JavaScript) -> Result<()> {
    match js {
        JavaScript::Npm(n) => publish_javascript_profile_npm_target(ctx.clone(), n).await?,
        JavaScript::Git(g) => publish_javascript_profile_git_target(ctx.clone(), g).await?,
    }
    Ok(())
}
