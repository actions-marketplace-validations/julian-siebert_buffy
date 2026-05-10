use tera::{Context as TeraContext, Tera};
use tokio::process::Command;

use crate::{
    configs::profiles::javascript::JavaScript,
    dependencies::protoc,
    error::{Error, Result},
    targets::context::Context,
};

const PACKAGE_JSON_TEMPLATE: &str = include_str!("templates/package.json.tera");

pub async fn generate_js_code(ctx: &Context, js: &JavaScript) -> Result<()> {
    let grpc = match js {
        JavaScript::Npm(n) => n.grpc,
        JavaScript::Git(g) => g.grpc,
    };

    ctx.pb.set_message("Generating JavaScript code...");
    let mut cmd = Command::new(protoc()?);
    cmd.arg(format!(
        "--js_out=import_style=commonjs,binary:{}",
        ctx.target_path.display()
    ))
    .arg(format!("--proto_path={}", ctx.source.path.display()));

    if grpc {
        cmd.arg(format!(
            "--grpc-web_out=import_style=commonjs,mode=grpcwebtext:{}",
            ctx.target_path.display()
        ));
    }

    cmd.args(ctx.proto_files());
    ctx.run(&mut cmd).await?;

    Ok(())
}

pub fn render_package_json(
    ctx: &Context,
    name: &str,
    repository: &str,
    homepage: Option<&str>,
    registry: Option<&str>,
    access: Option<&str>,
    grpc: bool,
) -> Result<String> {
    let mut tera = Tera::default();
    tera.add_raw_template("package.json", PACKAGE_JSON_TEMPLATE)
        .map_err(|e| Error::Internal(format!("Tera template error: {e}")))?;

    let author = ctx
        .package
        .authors
        .first()
        .map(|a| a.to_string())
        .unwrap_or_default();

    let mut tctx = TeraContext::new();
    tctx.insert("name", name);
    tctx.insert("description", &ctx.package.description);
    tctx.insert("version", &ctx.package.version.to_string());
    tctx.insert("license", &ctx.package.license);
    tctx.insert("repository", repository);
    tctx.insert(
        "homepage",
        homepage.unwrap_or_else(|| ctx.package.homepage.as_str()),
    );
    tctx.insert("author", &author);
    tctx.insert("grpc", &grpc);
    if let Some(r) = registry {
        tctx.insert("registry", r);
    }
    if let Some(a) = access {
        tctx.insert("access", a);
    }

    tera.render("package.json", &tctx)
        .map_err(|e| Error::Internal(format!("Tera render error: {e}")))
}
