use indicatif::ProgressBar;

use crate::{
    configs::{
        Package, Source,
        profiles::{
            NamedProfile,
            Profile::{Golang, Java, JavaScript, Kotlin, Rust, TypeScript},
        },
    },
    error::Result,
    targets::{
        context::Context,
        golang::{
            build_go_profile_target, check_go_profile_target, git::publish_go_profile_git_target,
            publish_go_profile_target,
        },
    },
};

pub mod context;
mod golang;
mod java;
mod javascript;
mod kotlin;
mod rust;
mod typescript;

pub async fn check_profile_target(ctx: Context) -> Result<()> {
    match ctx.profile.kind() {
        Golang(golang) => check_go_profile_target(ctx.clone(), golang).await?,
        Java(java) => {}
        Kotlin(kotlin) => {}
        JavaScript(java_script) => {}
        Rust(rust) => {}
        TypeScript(type_script) => {}
    };

    Ok(())
}

pub async fn build_profile_target(ctx: Context) -> Result<()> {
    match ctx.profile.kind() {
        Golang(golang) => build_go_profile_target(ctx.clone(), golang).await?,
        Java(java) => {}
        Kotlin(kotlin) => {}
        JavaScript(java_script) => {}
        Rust(rust) => {}
        TypeScript(type_script) => {}
    };

    Ok(())
}

pub async fn publish_profile_target(ctx: Context) -> Result<()> {
    match ctx.profile.kind() {
        Golang(golang) => publish_go_profile_target(ctx.clone(), golang).await?,
        Java(java) => {}
        Kotlin(kotlin) => {}
        JavaScript(java_script) => {}
        Rust(rust) => {}
        TypeScript(type_script) => {}
    };

    Ok(())
}
