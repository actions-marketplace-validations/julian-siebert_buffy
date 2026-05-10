use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tokio::process::Command;

use crate::{
    configs::profiles::rust::Rust, dependencies::protoc, error::Result, targets::context::Context,
};

pub async fn generate_rust_code(ctx: &Context, rust: &Rust) -> Result<()> {
    let src_dir = ctx.target_path.join("src");
    crate::io::create_dir_all(&src_dir)?;

    let grpc = match rust {
        Rust::Crate(c) => c.grpc,
        Rust::Git(g) => g.grpc,
    };

    ctx.pb
        .set_message("Generating Rust code from proto files...");

    let mut cmd = Command::new(protoc()?);
    cmd.arg(format!("--prost_out={}", src_dir.display()))
        .arg(format!("--proto_path={}", ctx.source.path.display()));
    if grpc {
        cmd.arg(format!("--tonic_out={}", src_dir.display()));
    }
    cmd.args(ctx.proto_files());

    ctx.run(&mut cmd).await?;
    Ok(())
}

pub async fn generate_lib_rs(ctx: &Context) -> Result<()> {
    ctx.pb.set_message("Generating lib.rs...");

    let src_dir = ctx.target_path.join("src");
    let files = collect_files_recursive(&src_dir)?;
    let lib_rs = build_lib_rs(&src_dir, &files);

    // clean up generated files, keep only lib.rs
    for entry in crate::io::read_dir(&src_dir)? {
        let path = entry?;
        if path.is_dir() {
            crate::io::remove_dir_all(&path)?;
        } else if path != src_dir.join("lib.rs") {
            std::fs::remove_file(&path).map_err(|e| crate::io::Error::Other {
                path: path.clone(),
                source: e,
            })?;
        }
    }

    crate::io::write(src_dir.join("lib.rs"), lib_rs)?;
    Ok(())
}

fn collect_files_recursive(dir: &Path) -> Result<HashMap<PathBuf, String>> {
    let mut files = HashMap::new();
    for entry in crate::io::read_dir(dir)? {
        let path = entry?;
        if path.is_dir() {
            files.extend(collect_files_recursive(&path)?);
        } else {
            let content = crate::io::read_to_string(&path)?;
            files.insert(path, content);
        }
    }
    Ok(files)
}

fn build_lib_rs(src_dir: &Path, files: &HashMap<PathBuf, String>) -> String {
    let resolve_content = |content: &str, _dir: &Path| -> String {
        let mut result = content.to_string();
        for (path, other_content) in files {
            if let Some(name) = path.file_name() {
                let name = name.to_string_lossy();
                result = result.replace(&format!("include!(\".{name}\");"), other_content);
                result = result.replace(&format!("include!(\"{name}\");"), other_content);
            }
        }
        result
    };

    fn build_module(
        dir: &Path,
        files: &HashMap<PathBuf, String>,
        resolve: &dyn Fn(&str, &Path) -> String,
    ) -> String {
        let mut output = String::new();

        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by_key(|e| e.path());

        for entry in &entries {
            let path = entry.path();
            if path.is_file() {
                let name = path.file_name().unwrap().to_string_lossy();
                if name == "lib.rs" || name.starts_with('.') || name.ends_with(".tonic.rs") {
                    continue;
                }
                if let Some(content) = files.get(&path) {
                    output.push_str(&resolve(content, path.parent().unwrap()));
                    output.push('\n');
                }
            }
        }

        for entry in &entries {
            let path = entry.path();
            if path.is_dir() {
                let mod_name = path.file_name().unwrap().to_string_lossy();
                let inner = build_module(&path, files, resolve);
                if !inner.trim().is_empty() {
                    output.push_str(&format!("pub mod {mod_name} {{\n"));
                    for line in inner.lines() {
                        output.push_str(&format!("    {line}\n"));
                    }
                    output.push_str("}\n\n");
                }
            }
        }

        output
    }

    build_module(src_dir, files, &resolve_content)
}
