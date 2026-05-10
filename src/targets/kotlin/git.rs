use console::style;

use crate::{
    configs::profiles::kotlin::Git,
    error::Result,
    git,
    targets::{
        context::Context,
        kotlin::helpers::{
            render_pom, resolve_kotlin_version, resolve_protobuf_version, verify_compile,
        },
    },
};

pub async fn build_kotlin_profile_git_target(ctx: Context, g: &Git) -> Result<()> {
    ctx.pb.set_message("Resolving versions...");
    let protobuf_version = resolve_protobuf_version(g.protobuf_version.as_deref()).await?;
    let kotlin_version = resolve_kotlin_version(g.kotlin_version.as_deref()).await?;

    ctx.pb.suspend(|| {
        eprintln!(
            "{} {} using protobuf-java {} + kotlin {}",
            style("[i]").cyan().bold(),
            style("KOTLIN").bold(),
            style(format!("v{protobuf_version}")).yellow(),
            style(format!("v{kotlin_version}")).yellow(),
        );
    });

    ctx.pb.set_message("Generating pom.xml...");
    let pom = render_pom(
        &ctx,
        &g.group_id,
        &g.artifact_id,
        &g.url,
        &g.scm,
        &protobuf_version,
        &kotlin_version,
        false,      // auto_publish irrelevant for git variant
        "uploaded", // wait_until irrelevant for git variant
    )?;
    crate::io::write(ctx.target_path.join("pom.xml"), pom)?;

    crate::gitignore::ensure_entries_in_gitignore(
        &ctx.target_path,
        &["target", ".gpg-key.asc", ".buffy-settings.xml"],
    )?;

    verify_compile(&ctx).await?;
    Ok(())
}

pub async fn publish_kotlin_profile_git_target(ctx: Context, g: &Git) -> Result<()> {
    let version = ctx.package.version.to_string();
    let tag = format!("v{version}");
    let remote = &g.remote;
    let branch = &g.branch;

    ctx.pb.set_message("Initializing git repository...");
    git!(ctx, "init", "-b", branch)?;
    git!(ctx, "config", "user.email", "buffy@localhost")?;
    git!(ctx, "config", "user.name", "buffy")?;

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

    ctx.pb
        .finish_with_message(format!("✓ Published {tag} → {remote}"));
    Ok(())
}
