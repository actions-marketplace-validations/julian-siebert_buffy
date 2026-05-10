use console::style;

use crate::{
    configs::profiles::rust::Git,
    error::Result,
    git,
    targets::{
        context::Context,
        rust::helpers::{render_cargo_toml, resolve_crate_version},
    },
};

pub async fn build_rust_profile_git_target(ctx: Context, g: &Git) -> Result<()> {
    ctx.pb.set_message("Resolving prost version...");
    let prost_version = resolve_crate_version("prost", g.prost_version.as_deref()).await?;

    let tonic_version = if g.grpc {
        Some(resolve_crate_version("tonic", g.tonic_version.as_deref()).await?)
    } else {
        None
    };

    ctx.pb.set_message("Generating Cargo.toml...");
    let cargo_toml = render_cargo_toml(
        &ctx,
        &g.name.clone().unwrap_or(ctx.package.name.clone()),
        &g.edition,
        &g.repository,
        &g.documentation,
        &prost_version,
        tonic_version.as_deref(),
        g.grpc,
    )?;
    crate::io::write(ctx.target_path.join("Cargo.toml"), cargo_toml)?;

    crate::gitignore::ensure_entries_in_gitignore(&ctx.target_path, &["target", "Cargo.lock"])?;

    Ok(())
}

pub async fn publish_rust_profile_git_target(ctx: Context, g: &Git) -> Result<()> {
    let version = ctx.package.version.to_string();
    let tag = format!("v{version}");
    let remote = &g.remote;
    let branch = &g.branch;

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
        for file in &g.keep {
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
    }

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

    ctx.finish_publish(&tag, remote);

    Ok(())
}
