use console::style;
use tokio::process::Command;

use crate::{
    configs::profiles::golang::{Git, Golang},
    dependencies::{git as git_dep, protoc, protoc_gen_go_grpc},
    error::Result,
    git,
    targets::context::Context,
};

pub async fn check_go_profile_git_target(ctx: Context, _golang: &Golang, git: &Git) -> Result<()> {
    ctx.pb.set_message("Checking git...");
    git_dep()?;

    if git.grpc {
        ctx.pb.set_message("Checking protoc-gen-go-grpc...");
        protoc_gen_go_grpc()?;
    }
    Ok(())
}

pub async fn build_go_profile_git_target(ctx: Context, _golang: &Golang, git: &Git) -> Result<()> {
    if git.grpc {
        build_go_profile_git_grpc_target(ctx, git).await?;
    }

    Ok(())
}

async fn build_go_profile_git_grpc_target(ctx: Context, git: &Git) -> Result<()> {
    ctx.pb.set_message("Generating gRPC code...");

    let mut cmd = Command::new(protoc()?);

    cmd.arg(format!("--go-grpc_out={}", ctx.target_path.display()))
        .arg(format!("--go-grpc_opt=module={}", &git.module))
        .arg(format!("--proto_path={}", ctx.source.path.display()))
        .args(ctx.proto_files());

    ctx.run(&mut cmd).await?;

    Ok(())
}

pub async fn publish_go_profile_git_target(
    ctx: Context,
    _golang: &Golang,
    git: &Git,
) -> Result<()> {
    let version = ctx.package.version.to_string();
    let tag = format!("v{version}");
    let remote = &git.remote;
    let branch = &git.branch;

    ctx.pb.set_message("Initializing git repository...");
    git!(ctx, "init", "-b", branch)?;

    ctx.pb.set_message("Configuring remote...");
    if git!(ctx, "remote", "add", "origin", remote).is_err() {
        git!(ctx, "remote", "set-url", "origin", remote)?;
    }

    ctx.pb.set_message("Fetching existing files from remote...");
    let fetch_result = git!(
        ctx,
        env: [("GIT_TERMINAL_PROMPT", "0")],
        "fetch", "origin", branch
    );

    if fetch_result.is_ok() {
        for file in &git.keep {
            let result = git!(
                ctx,
                "checkout",
                &format!("origin/{branch}"),
                "--",
                file.as_str()
            );

            if result.is_err() {
                ctx.pb.suspend(|| {
                    eprintln!(
                        "{} {} not found on remote, skipping",
                        style("[~]").yellow().bold(),
                        style(file).dim(),
                    );
                });
            }
        }
    } else {
        ctx.pb.suspend(|| {
            eprintln!(
                "{} remote branch `{branch}` does not exist yet, will be created on push",
                style("[~]").yellow().bold(),
            );
        });
    }

    ctx.pb.set_message("Adding files and commiting...");
    git!(ctx, "add", ".")?;
    git!(ctx, "commit", "-m", &format!("release {tag}"))?;

    ctx.pb.set_message(format!("Tagging {tag}..."));
    git!(ctx, "tag", "-f", &tag)?;

    ctx.pb.set_message(format!("Pushing {tag} to {branch}..."));
    git!(
        ctx,
        env: [("GIT_TERMINAL_PROMPT", "0")],
        "push", "--force", "origin", &format!("HEAD:{branch}")
    )?;

    git!(
        ctx,
        env: [("GIT_TERMINAL_PROMPT", "0")],
        "push", "--force", "origin", "--tags"
    )?;

    ctx.finish_publish(&tag, &remote);

    Ok(())
}
