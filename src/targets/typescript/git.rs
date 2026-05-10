use console::style;

use crate::{
    configs::profiles::typescript::Git,
    error::Result,
    git,
    targets::{
        context::Context,
        typescript::helpers::{install_and_build, render_package_json, render_tsconfig},
    },
};

pub async fn build_typescript_profile_git_target(ctx: Context, g: &Git) -> Result<()> {
    ctx.pb.set_message("Generating package.json...");
    let pkg = render_package_json(
        &ctx,
        &g.name,
        &g.repository,
        g.homepage.as_deref(),
        None,
        None,
        g.grpc,
    )?;
    crate::io::write(ctx.target_path.join("package.json"), pkg)?;

    ctx.pb.set_message("Generating tsconfig.json...");
    let tsconfig = render_tsconfig()?;
    crate::io::write(ctx.target_path.join("tsconfig.json"), tsconfig)?;

    crate::gitignore::ensure_entries_in_gitignore(&ctx.target_path, &["node_modules", "dist"])?;

    install_and_build(&ctx).await?;

    Ok(())
}

pub async fn publish_typescript_profile_git_target(ctx: Context, g: &Git) -> Result<()> {
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
