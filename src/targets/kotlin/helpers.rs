use tera::{Context as TeraContext, Tera};
use tokio::process::Command;

use crate::{
    configs::profiles::kotlin::Scm,
    dependencies::protoc,
    error::{Error, Result},
    license::resolve_licenses,
    targets::context::Context,
};

const POM_TEMPLATE: &str = include_str!("templates/pom.xml.tera");

pub async fn generate_kotlin_code(ctx: &Context) -> Result<()> {
    let java_dir = ctx.target_path.join("src/main/java");
    crate::io::create_dir_all(&java_dir)?;

    // empty Kotlin source dir for any user-added Kotlin files
    let kotlin_dir = ctx.target_path.join("src/main/kotlin");
    crate::io::create_dir_all(&kotlin_dir)?;

    ctx.pb
        .set_message("Generating Java code (consumed by Kotlin)...");
    let mut cmd = Command::new(protoc()?);
    cmd.arg(format!("--java_out={}", java_dir.display()))
        .arg(format!("--proto_path={}", ctx.source.path.display()))
        .args(ctx.proto_files());
    ctx.run(&mut cmd).await?;
    Ok(())
}

pub async fn resolve_protobuf_version(configured: Option<&str>) -> Result<String> {
    if let Some(v) = configured {
        return Ok(v.to_string());
    }

    let client = reqwest::Client::builder()
        .user_agent("buffy-build-tool/1.0")
        .build()
        .map_err(|e| Error::Internal(format!("HTTP client error: {e}")))?;

    let body = client
        .get("https://search.maven.org/solrsearch/select?q=g:com.google.protobuf+AND+a:protobuf-java&rows=1&wt=json")
        .send()
        .await
        .map_err(|e| Error::Internal(format!("Maven Central API unreachable: {e}")))?
        .text()
        .await
        .map_err(|e| Error::Internal(format!("Maven Central API read error: {e}")))?;

    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| Error::Internal(format!("Maven Central API parse error: {e}")))?;

    json["response"]["docs"][0]["latestVersion"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| {
            Error::Internal(
                "Could not resolve protobuf-java version from Maven Central.\n\
                 Pin it manually in your profile."
                    .into(),
            )
        })
}

pub async fn resolve_kotlin_version(configured: Option<&str>) -> Result<String> {
    if let Some(v) = configured {
        return Ok(v.to_string());
    }

    let client = reqwest::Client::builder()
        .user_agent("buffy-build-tool/1.0")
        .build()
        .map_err(|e| Error::Internal(format!("HTTP client error: {e}")))?;

    let body = client
        .get("https://search.maven.org/solrsearch/select?q=g:org.jetbrains.kotlin+AND+a:kotlin-stdlib&rows=1&wt=json")
        .send()
        .await
        .map_err(|e| Error::Internal(format!("Maven Central API unreachable: {e}")))?
        .text()
        .await
        .map_err(|e| Error::Internal(format!("Maven Central API read error: {e}")))?;

    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| Error::Internal(format!("Maven Central API parse error: {e}")))?;

    json["response"]["docs"][0]["latestVersion"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| {
            Error::Internal(
                "Could not resolve kotlin-stdlib version from Maven Central.\n\
                 Pin it manually in your profile."
                    .into(),
            )
        })
}

#[derive(serde::Serialize)]
struct AuthorView<'a> {
    name: &'a str,
    email: Option<&'a str>,
}

pub fn render_pom(
    ctx: &Context,
    group_id: &str,
    artifact_id: &str,
    url: &str,
    scm: &Scm,
    protobuf_version: &str,
    kotlin_version: &str,
    auto_publish: bool,
    wait_until: &str,
) -> Result<String> {
    let mut tera = Tera::default();
    tera.add_raw_template("pom.xml", POM_TEMPLATE)
        .map_err(|e| Error::Internal(format!("Tera template error: {e}")))?;
    tera.autoescape_on(vec![]);

    let licenses = resolve_licenses(&ctx.package.license)?;
    let authors_view: Vec<AuthorView> = ctx
        .package
        .authors
        .iter()
        .map(|a| AuthorView {
            name: &a.name,
            email: a.email.as_deref(),
        })
        .collect();

    let mut tctx = TeraContext::new();
    tctx.insert("group_id", group_id);
    tctx.insert("artifact_id", artifact_id);
    tctx.insert("version", &ctx.package.version.to_string());
    tctx.insert("description", &ctx.package.description);
    tctx.insert("url", url);
    tctx.insert("licenses", &licenses);
    tctx.insert("authors", &authors_view);
    tctx.insert("scm_connection", &scm.connection);
    tctx.insert("scm_url", &scm.url);
    tctx.insert("protobuf_version", protobuf_version);
    tctx.insert("kotlin_version", kotlin_version);
    tctx.insert("auto_publish", &auto_publish);
    tctx.insert("wait_until", wait_until);
    tctx.insert("gpg_keyname", &env_nonempty("GPG_KEY_ID").is_some());

    tera.render("pom.xml", &tctx)
        .map_err(|e| Error::Internal(format!("Tera render error: {e}")))
}

pub async fn verify_compile(ctx: &Context) -> Result<()> {
    ctx.pb.set_message("Verifying with mvn compile...");
    let mut cmd = Command::new("mvn");
    cmd.args(["compile", "-q", "--no-transfer-progress"])
        .current_dir(&ctx.target_path);
    ctx.run(&mut cmd).await?;
    Ok(())
}

pub fn env_nonempty(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|s| !s.is_empty())
}
