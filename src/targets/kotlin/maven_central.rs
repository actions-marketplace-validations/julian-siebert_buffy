use console::style;
use tokio::process::Command;

use crate::{
    configs::profiles::kotlin::MavenCentral,
    error::{Error, Result},
    targets::{
        context::Context,
        kotlin::helpers::{
            env_nonempty, render_pom, resolve_kotlin_version, resolve_protobuf_version,
            verify_compile,
        },
    },
};

pub async fn build_kotlin_profile_maven_central_target(
    ctx: Context,
    m: &MavenCentral,
) -> Result<()> {
    ctx.pb.set_message("Resolving versions...");
    let protobuf_version = resolve_protobuf_version(m.protobuf_version.as_deref()).await?;
    let kotlin_version = resolve_kotlin_version(m.kotlin_version.as_deref()).await?;

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
        &m.group_id,
        &m.artifact_id,
        &m.url,
        &m.scm,
        &protobuf_version,
        &kotlin_version,
        m.auto_publish,
        &m.wait_until,
    )?;
    crate::io::write(ctx.target_path.join("pom.xml"), pom)?;

    verify_compile(&ctx).await?;
    Ok(())
}

pub async fn publish_kotlin_profile_maven_central_target(
    ctx: Context,
    m: &MavenCentral,
) -> Result<()> {
    let username = env_nonempty("MAVEN_USERNAME");
    let password = env_nonempty("MAVEN_PASSWORD");
    if username.is_none() || password.is_none() {
        return Err(Error::MissingEnv {
            name: "MAVEN_USERNAME / MAVEN_PASSWORD".into(),
            hint: indoc::indoc! {"
                Set these environment variables before publishing:
                MAVEN_USERNAME   – Maven Central username (portal.central.sonatype.com)
                MAVEN_PASSWORD   – Maven Central token
                GPG_KEY_ID       – GPG key ID used for signing
                GPG_PASSPHRASE   – GPG key passphrase
            "}
            .into(),
        });
    }
    let username = username.unwrap();
    let password = password.unwrap();

    let settings_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<settings xmlns="http://maven.apache.org/SETTINGS/1.0.0">
  <servers>
    <server>
      <id>central</id>
      <username>{username}</username>
      <password>{password}</password>
    </server>
  </servers>
</settings>
"#
    );
    let settings_path = ctx.target_path.join(".buffy-settings.xml");
    crate::io::write(&settings_path, settings_xml)?;
    let abs_settings_path = settings_path
        .canonicalize()
        .map_err(|e| crate::io::from_io(&settings_path, e))?;

    if let Ok(key) = std::env::var("GPG_PRIVATE_KEY") {
        ctx.pb.set_message("Importing GPG key...");
        let key_file = ctx.target_path.join(".gpg-key.asc");
        crate::io::write(&key_file, key)?;
        let abs_key_file = key_file
            .canonicalize()
            .map_err(|e| crate::io::from_io(&key_file, e))?;

        let mut cmd = Command::new("gpg");
        cmd.args(["--batch", "--import"]).arg(&abs_key_file);
        ctx.run(&mut cmd).await?;
        std::fs::remove_file(&key_file).ok();
    }

    let version = ctx.package.version.to_string();
    ctx.pb.set_message(format!(
        "Publishing {}:{} v{version} to Maven Central...",
        m.group_id, m.artifact_id
    ));

    let mut args = vec![
        "deploy".to_string(),
        "--batch-mode".to_string(),
        "--no-transfer-progress".to_string(),
        "-s".to_string(),
        abs_settings_path.to_string_lossy().into_owned(),
    ];
    if let Some(key_id) = env_nonempty("GPG_KEY_ID") {
        args.push(format!("-Dgpg.keyname={key_id}"));
    }

    let mut cmd = Command::new("mvn");
    cmd.args(&args).current_dir(&ctx.target_path);

    if let Some(passphrase) = env_nonempty("GPG_PASSPHRASE") {
        cmd.env("MAVEN_GPG_PASSPHRASE", passphrase);
    }

    let result = ctx.run(&mut cmd).await;
    std::fs::remove_file(&settings_path).ok();
    result?;

    ctx.finish_publish(&version, "Maven Central");

    Ok(())
}
