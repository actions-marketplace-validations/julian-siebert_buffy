use tokio::process::Command;

use crate::{
    configs::profiles::typescript::Npm,
    dependencies::npm,
    error::{Error, Result},
    targets::{
        context::Context,
        typescript::helpers::{install_and_build, render_package_json, render_tsconfig},
    },
};

pub async fn build_typescript_profile_npm_target(ctx: Context, n: &Npm) -> Result<()> {
    ctx.pb.set_message("Generating package.json...");
    let pkg = render_package_json(
        &ctx,
        &n.name,
        &n.repository,
        n.homepage.as_deref(),
        Some(&n.registry),
        Some(&n.access),
        n.grpc,
    )?;
    crate::io::write(ctx.target_path.join("package.json"), pkg)?;

    ctx.pb.set_message("Generating tsconfig.json...");
    let tsconfig = render_tsconfig()?;
    crate::io::write(ctx.target_path.join("tsconfig.json"), tsconfig)?;

    install_and_build(&ctx).await?;

    // dry-run publish
    ctx.pb.set_message("Verifying package layout...");
    let mut cmd = Command::new(npm()?);
    cmd.args(["publish", "--dry-run", "--no-audit"])
        .current_dir(&ctx.target_path);
    ctx.run(&mut cmd).await?;

    Ok(())
}

pub async fn publish_typescript_profile_npm_target(ctx: Context, n: &Npm) -> Result<()> {
    if std::env::var("NPM_TOKEN").is_err() {
        return Err(Error::MissingEnv {
            name: "NPM_TOKEN".into(),
            hint: indoc::indoc! {"
                Set this environment variable before publishing:

                NPM_TOKEN – Auth token from `npm token create` or your registry UI

                Alternatively, run `npm login` and ensure ~/.npmrc has the auth set.
            "}
            .into(),
        });
    }

    let registry_host = n
        .registry
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/');
    let npmrc = format!(
        "//{registry_host}/:_authToken={}\nregistry={}\n",
        std::env::var("NPM_TOKEN").unwrap(),
        n.registry,
    );
    crate::io::write(ctx.target_path.join(".npmrc"), npmrc)?;

    let version = ctx.package.version.to_string();

    let mut cmd = Command::new(npm()?);
    cmd.args(["publish", "--access", &n.access])
        .current_dir(&ctx.target_path);
    ctx.run(&mut cmd).await?;

    ctx.finish_publish(&version, &n.registry);

    Ok(())
}
