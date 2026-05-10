use tera::{Context as TeraContext, Tera};

use crate::{
    error::{Error, Result},
    targets::context::Context,
};

const CARGO_TOML_TEMPLATE: &str = include_str!("templates/Cargo.toml.tera");

pub async fn resolve_crate_version(name: &str, configured: Option<&str>) -> Result<String> {
    if let Some(v) = configured {
        return Ok(v.to_string());
    }

    let client = reqwest::Client::builder()
        .user_agent("buffy-build-tool/1.0")
        .build()
        .map_err(|e| Error::Internal(format!("HTTP client error: {e}")))?;

    let url = format!("https://crates.io/api/v1/crates/{name}");
    let body = client
        .get(&url)
        .send()
        .await
        .map_err(|e| Error::Internal(format!("crates.io API unreachable: {e}")))?
        .text()
        .await
        .map_err(|e| Error::Internal(format!("crates.io API read error: {e}")))?;

    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| Error::Internal(format!("crates.io API parse error: {e}")))?;

    json["crate"]["newest_version"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| {
            Error::Internal(format!(
                "Could not resolve {name} version from crates.io.\n\
                 Pin it manually in your profile."
            ))
        })
}

pub fn render_cargo_toml(
    ctx: &Context,
    name: &str,
    edition: &str,
    repository: &str,
    documentation: &str,
    prost_version: &str,
    tonic_version: Option<&str>,
    grpc: bool,
) -> Result<String> {
    let mut tera = Tera::default();
    tera.add_raw_template("Cargo.toml", CARGO_TOML_TEMPLATE)
        .map_err(|e| Error::Internal(format!("Tera template error: {e}")))?;

    let lib_name = name.replace('-', "_");
    let authors_toml: Vec<String> = ctx
        .package
        .authors
        .iter()
        .map(|a| format!("\"{a}\""))
        .collect();

    let mut tctx = TeraContext::new();
    tctx.insert("name", name);
    tctx.insert("description", &ctx.package.description);
    tctx.insert("lib_name", &lib_name);
    tctx.insert("version", &ctx.package.version.to_string());
    tctx.insert("edition", edition);
    tctx.insert("authors", &authors_toml);
    tctx.insert("license", &ctx.package.license);
    tctx.insert("documentation", documentation);
    tctx.insert("homepage", &ctx.package.homepage);
    tctx.insert("repository", repository);
    tctx.insert("prost_version", prost_version);
    tctx.insert("grpc", &grpc);
    tctx.insert("tonic_version", &tonic_version.unwrap_or(""));

    tera.render("Cargo.toml", &tctx)
        .map_err(|e| Error::Internal(format!("Tera render error: {e}")))
}
