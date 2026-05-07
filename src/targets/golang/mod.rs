use tokio::process::Command;

use crate::{
    configs::profiles::golang::Golang,
    dependencies::{go, protoc, protoc_gen_go},
    error::Result,
    license::resolve_licenses,
    targets::{
        context::Context,
        golang::git::{
            build_go_profile_git_target, check_go_profile_git_target, publish_go_profile_git_target,
        },
    },
};

pub mod git;

pub async fn check_go_profile_target(ctx: Context, golang: &Golang) -> Result<()> {
    ctx.pb.set_message("Checking protoc...");
    protoc()?;

    ctx.pb.set_message("Checking protoc-gen-go...");
    protoc_gen_go()?;

    ctx.pb.set_message("Checking go toolchain...");
    go()?;

    match golang {
        Golang::Git(git) => {
            check_go_profile_git_target(ctx.clone(), golang, git).await?;
        }
    };

    ctx.finish_check();

    Ok(())
}

pub async fn build_go_profile_target(ctx: Context, golang: &Golang) -> Result<()> {
    let module = match golang {
        Golang::Git(git) => &git.module,
    };

    ctx.pb.set_message("Generating Golang code...");
    let mut cmd = Command::new(protoc()?);
    cmd.arg(format!("--go_out={}", ctx.target_path.display()))
        .arg(format!("--go_opt=module={}", module))
        .arg(format!("--proto_path={}", ctx.source.path.display()))
        .args(ctx.proto_files());
    ctx.run(&mut cmd).await?;

    match golang {
        Golang::Git(git) => {
            build_go_profile_git_target(ctx.clone(), golang, git).await?;
        }
    };

    ctx.pb.set_message("Initializing go module...");
    let mut cmd = Command::new("go");
    cmd.args(["mod", "init", module])
        .current_dir(&ctx.target_path);
    ctx.run(&mut cmd).await?;

    ctx.pb.set_message("Tidying go module...");
    let mut cmd = Command::new("go");
    cmd.args(["mod", "tidy"]).current_dir(&ctx.target_path);
    ctx.run(&mut cmd).await?;

    ctx.pb.set_message("Writing LICENSE file(s)...");
    let licenses = resolve_licenses(&ctx.package.license)?;
    match licenses.as_slice() {
        [] => unreachable!("resolve_licenses returns at least one license"),
        [single] => {
            crate::io::write(ctx.target_path.join("LICENSE"), &single.text)?;
        }
        multiple => {
            let mut index = format!(
                "This project is licensed under: {}\n\n\
                 The full text of each license is provided in the corresponding \
                 file listed below.\n\n",
                ctx.package.license,
            );
            for lic in multiple {
                let filename = format!("LICENSE-{}", lic.id);
                crate::io::write(ctx.target_path.join(&filename), &lic.text)?;
                index.push_str(&format!("- {}: see {}\n", lic.name, filename));
            }
            crate::io::write(ctx.target_path.join("LICENSE"), index)?;
        }
    }

    if !ctx.package.authors.is_empty() {
        ctx.pb.set_message("Writing AUTHORS file...");
        let authors_file = ctx
            .package
            .authors
            .iter()
            .map(|a| a.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        crate::io::write(ctx.target_path.join("AUTHORS"), authors_file + "\n")?;
    }

    ctx.pb.set_message("Verifying go build...");
    let mut cmd = Command::new("go");
    cmd.args(["build", "./..."]).current_dir(&ctx.target_path);
    ctx.run(&mut cmd).await?;

    ctx.finish_build();

    Ok(())
}

pub async fn publish_go_profile_target(ctx: Context, golang: &Golang) -> Result<()> {
    match golang {
        Golang::Git(git) => {
            publish_go_profile_git_target(ctx.clone(), golang, git).await?;
        }
    };

    Ok(())
}
