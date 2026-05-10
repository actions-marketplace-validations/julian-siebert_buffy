use crate::{
    configs::profiles::kotlin::Kotlin,
    dependencies::{java as java_dep, maven, protoc},
    error::Result,
    targets::{
        context::Context,
        kotlin::{
            git::{build_kotlin_profile_git_target, publish_kotlin_profile_git_target},
            helpers::generate_kotlin_code,
            maven_central::{
                build_kotlin_profile_maven_central_target,
                publish_kotlin_profile_maven_central_target,
            },
        },
    },
};

pub mod git;
mod helpers;
pub mod maven_central;

pub async fn check_kotlin_profile_target(ctx: Context, kotlin: &Kotlin) -> Result<()> {
    ctx.pb.set_message("Checking protoc...");
    protoc()?;

    ctx.pb.set_message("Checking java...");
    java_dep()?;

    ctx.pb.set_message("Checking maven...");
    maven()?;

    match kotlin {
        Kotlin::MavenCentral(_) => {
            ctx.pb.set_message("Checking gpg...");
            crate::dependencies::gpg()?;
        }
        Kotlin::Git(_) => {
            ctx.pb.set_message("Checking git...");
            crate::dependencies::git()?;
        }
    }

    Ok(())
}

pub async fn build_kotlin_profile_target(ctx: Context, kotlin: &Kotlin) -> Result<()> {
    generate_kotlin_code(&ctx).await?;

    match kotlin {
        Kotlin::MavenCentral(m) => {
            build_kotlin_profile_maven_central_target(ctx.clone(), m).await?
        }
        Kotlin::Git(g) => build_kotlin_profile_git_target(ctx.clone(), g).await?,
    }

    Ok(())
}

pub async fn publish_kotlin_profile_target(ctx: Context, kotlin: &Kotlin) -> Result<()> {
    match kotlin {
        Kotlin::MavenCentral(m) => {
            publish_kotlin_profile_maven_central_target(ctx.clone(), m).await?
        }
        Kotlin::Git(g) => publish_kotlin_profile_git_target(ctx.clone(), g).await?,
    }
    Ok(())
}
