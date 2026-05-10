use crate::{
    configs::profiles::java::Java,
    dependencies::{java as java_dep, maven, protoc},
    error::Result,
    targets::{
        context::Context,
        java::{
            git::{build_java_profile_git_target, publish_java_profile_git_target},
            maven_central::{build_java_profile_maven_target, publish_java_profile_maven_target},
        },
    },
};

pub mod git;
mod helpers;
pub mod maven_central;

pub async fn check_java_profile_target(ctx: Context, java: &Java) -> Result<()> {
    ctx.pb.set_message("Checking protoc...");
    protoc()?;

    ctx.pb.set_message("Checking java...");
    java_dep()?;

    ctx.pb.set_message("Checking maven...");
    maven()?;

    match java {
        Java::MavenCentral(_) => {
            ctx.pb.set_message("Checking gpg...");
            crate::dependencies::gpg()?;
        }
        Java::Git(_) => {
            ctx.pb.set_message("Checking git...");
            crate::dependencies::git()?;
        }
    }

    ctx.finish_check();

    Ok(())
}

pub async fn build_java_profile_target(ctx: Context, java: &Java) -> Result<()> {
    helpers::generate_java_code(&ctx).await?;

    match java {
        Java::MavenCentral(m) => build_java_profile_maven_target(ctx.clone(), m).await?,
        Java::Git(g) => build_java_profile_git_target(ctx.clone(), g).await?,
    }

    ctx.finish_build();

    Ok(())
}

pub async fn publish_java_profile_target(ctx: Context, java: &Java) -> Result<()> {
    match java {
        Java::MavenCentral(m) => publish_java_profile_maven_target(ctx.clone(), m).await?,
        Java::Git(g) => publish_java_profile_git_target(ctx.clone(), g).await?,
    }
    Ok(())
}
