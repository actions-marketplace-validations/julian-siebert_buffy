use console::style;
use tokio::process::Command;

use crate::{
    configs::profiles::javascript::Git,
    dependencies::npm,
    error::Result,
    git,
    targets::{context::Context, javascript::helpers::render_package_json},
};

pub async fn build_javascript_profile_git_target(ctx: Context, g: &Git) -> Result<()> {
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

    crate::gitignore::ensure_entries_in_gitignore(&ctx.target_path, &["node_modules"])?;

    ctx.pb.set_message("Validating package.json...");
    let mut cmd = Command::new("npm");
    cmd.args(["pkg", "fix"]).current_dir(&ctx.target_path);
    ctx.run(&mut cmd).await?;

    // install deps to validate they resolve
    ctx.pb.set_message("Installing dependencies...");
    let mut cmd = Command::new(npm()?);
    cmd.args(["install", "--no-audit", "--no-fund", "--silent"])
        .current_dir(&ctx.target_path);
    ctx.run(&mut cmd).await?;

    // dry-run publish to catch packaging issues
    ctx.pb.set_message("Verifying package layout...");
    let mut cmd = Command::new(npm()?);
    cmd.args(["publish", "--dry-run", "--no-audit"])
        .current_dir(&ctx.target_path);
    ctx.run(&mut cmd).await?;

    Ok(())
}

pub async fn publish_javascript_profile_git_target(ctx: Context, g: &Git) -> Result<()> {
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
