use std::path::PathBuf;

use tera::{Context as TeraContext, Tera};
use tokio::process::Command;

use crate::{
    configs::profiles::typescript::TypeScript,
    dependencies::{npm, protoc},
    error::{Error, Result},
    targets::context::Context,
};

const PACKAGE_JSON_TEMPLATE: &str = include_str!("templates/package.json.tera");
const TSCONFIG_TEMPLATE: &str = include_str!("templates/tsconfig.json.tera");
const INDEX_TS_TEMPLATE: &str = include_str!("templates/index.ts.tera");

pub async fn generate_ts_code(ctx: &Context, ts: &TypeScript) -> Result<()> {
    let src_dir = ctx.target_path.join("src");
    crate::io::create_dir_all(&src_dir)?;

    let grpc = match ts {
        TypeScript::Npm(n) => n.grpc,
        TypeScript::Git(g) => g.grpc,
    };

    ctx.pb.set_message("Generating TypeScript code...");

    // ts-proto options
    let mut ts_opts = vec!["esModuleInterop=true".to_string()];
    if grpc {
        ts_opts.push("outputServices=nice-grpc".to_string());
        ts_opts.push("outputServices=generic-definitions".to_string());
    }

    let mut cmd = Command::new(protoc()?);
    cmd.arg(format!("--ts_proto_out={}", src_dir.display()))
        .arg(format!("--ts_proto_opt={}", ts_opts.join(",")))
        .arg(format!("--proto_path={}", ctx.source.path.display()))
        .args(ctx.proto_files());

    ctx.run(&mut cmd).await?;
    Ok(())
}

pub async fn generate_index_ts(ctx: &Context) -> Result<()> {
    ctx.pb.set_message("Generating index.ts...");

    let src_dir = ctx.target_path.join("src");
    let ts_files = collect_ts_files(&src_dir)?;

    // strip src/ prefix and .ts suffix for re-exports
    let module_paths: Vec<String> = ts_files
        .iter()
        .filter_map(|p| {
            let rel = p.strip_prefix(&src_dir).ok()?;
            let s = rel.to_string_lossy();
            let stem = s.strip_suffix(".ts")?;
            // skip index.ts itself (will be overwritten)
            if stem == "index" {
                return None;
            }
            Some(stem.to_string())
        })
        .collect();

    let mut tera = Tera::default();
    tera.add_raw_template("index.ts", INDEX_TS_TEMPLATE)
        .map_err(|e| Error::Internal(format!("Tera template error: {e}")))?;

    let mut tctx = TeraContext::new();
    tctx.insert("files", &module_paths);

    let rendered = tera
        .render("index.ts", &tctx)
        .map_err(|e| Error::Internal(format!("Tera render error: {e}")))?;

    crate::io::write(src_dir.join("index.ts"), rendered)?;
    Ok(())
}

fn collect_ts_files(dir: &std::path::Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    collect_recursive(dir, &mut out)?;
    Ok(out)
}

fn collect_recursive(dir: &std::path::Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in crate::io::read_dir(dir)? {
        let path = entry?;
        if path.is_dir() {
            collect_recursive(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("ts") {
            out.push(path);
        }
    }
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

pub fn render_tsconfig() -> Result<String> {
    let mut tera = Tera::default();
    tera.add_raw_template("tsconfig.json", TSCONFIG_TEMPLATE)
        .map_err(|e| Error::Internal(format!("Tera template error: {e}")))?;
    tera.render("tsconfig.json", &TeraContext::new())
        .map_err(|e| Error::Internal(format!("Tera render error: {e}")))
}

pub async fn install_and_build(ctx: &Context) -> Result<()> {
    ctx.pb.set_message("Validating package.json...");
    let mut cmd = Command::new(npm()?);
    cmd.args(["pkg", "fix"]).current_dir(&ctx.target_path);
    ctx.run(&mut cmd).await?;

    ctx.pb.set_message("Installing dependencies...");
    let mut cmd = Command::new(npm()?);
    cmd.args(["install", "--no-audit", "--no-fund"])
        .current_dir(&ctx.target_path);
    ctx.run(&mut cmd).await?;

    ctx.pb.set_message("Compiling TypeScript...");
    let mut cmd = Command::new(npm()?);
    cmd.args(["run", "build"]).current_dir(&ctx.target_path);
    ctx.run(&mut cmd).await?;

    Ok(())
}
