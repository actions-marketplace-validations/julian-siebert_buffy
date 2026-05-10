use console::style;

use crate::{
    configs::profiles::java::Git,
    error::Result,
    git,
    targets::{
        context::Context,
        java::helpers::{render_pom, resolve_protobuf_version, verify_compile},
    },
};

pub async fn build_java_profile_git_target(ctx: Context, g: &Git) -> Result<()> {
    ctx.pb.set_message("Resolving protobuf-java version...");
    let protobuf_version = resolve_protobuf_version(g.protobuf_version.as_deref()).await?;

    ctx.pb.suspend(|| {
        eprintln!(
            "{} {} using protobuf-java {}",
            style("[i]").cyan().bold(),
            style("JAVA").bold(),
            style(format!("v{}", &protobuf_version)).yellow(),
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
        false,
        "uploaded",
    )?;
    crate::io::write(ctx.target_path.join("pom.xml"), pom)?;

    crate::gitignore::ensure_entries_in_gitignore(&ctx.target_path, &["target", ".gpg-key.asc"])?;

    verify_compile(&ctx).await?;

    Ok(())
}

pub async fn publish_java_profile_git_target(ctx: Context, g: &Git) -> Result<()> {
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
