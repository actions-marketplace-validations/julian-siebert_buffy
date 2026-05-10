use tokio::process::Command;

use crate::{
    configs::profiles::rust::Crate,
    dependencies::cargo,
    error::{Error, Result},
    targets::{
        context::Context,
        rust::helpers::{render_cargo_toml, resolve_crate_version},
    },
};

pub async fn build_rust_profile_crate_target(ctx: Context, c: &Crate) -> Result<()> {
    ctx.pb.set_message("Resolving prost version...");
    let prost_version = resolve_crate_version("prost", c.prost_version.as_deref()).await?;

    let tonic_version = if c.grpc {
        Some(resolve_crate_version("tonic", c.tonic_version.as_deref()).await?)
    } else {
        None
    };

    ctx.pb.set_message("Generating Cargo.toml...");
    let cargo_toml = render_cargo_toml(
        &ctx,
        &c.name.clone().unwrap_or(ctx.package.name.clone()),
        &c.edition,
        &c.repository,
        &c.documentation,
        &prost_version,
        tonic_version.as_deref(),
        c.grpc,
    )?;
    crate::io::write(ctx.target_path.join("Cargo.toml"), cargo_toml)?;

    Ok(())
}

pub async fn publish_rust_profile_crate_target(ctx: Context, c: &Crate) -> Result<()> {
    if std::env::var("CARGO_REGISTRY_TOKEN").is_err() {
        return Err(Error::MissingEnv {
            name: "CARGO_REGISTRY_TOKEN".into(),
            hint: indoc::indoc! {"
                Set this environment variable before publishing:

                CARGO_REGISTRY_TOKEN – API token from https://crates.io/me
                                       or your self-hosted registry
            "}
            .into(),
        });
    }

    let version = ctx.package.version.to_string();
    ctx.pb.set_message(format!(
        "Publishing {} v{version} to {}...",
        c.name.clone().unwrap_or(ctx.package.name.clone()),
        c.registry
    ));

    let mut args = vec!["publish", "--no-verify"];
    if c.registry != "crates-io" {
        args.push("--registry");
        args.push(&c.registry);
    }

    let mut cmd = Command::new(cargo()?);
    cmd.args(&args).current_dir(&ctx.target_path);
    ctx.run(&mut cmd).await?;

    ctx.finish_publish(&version, &c.registry);

    Ok(())
}
