use std::sync::Arc;

use clap::Parser;
use console::style;
use indicatif::{MultiProgress, ProgressBar};
use miette::Diagnostic;
use tokio::task::JoinSet;

use crate::{
    cli::Cli,
    configs::{read_main, read_profiles},
    error::{Error, Result},
    gitignore::ensure_target_in_gitignore,
    targets::{
        build_profile_target, check_profile_target, context::Context, publish_profile_target,
    },
};

mod cli;
pub mod configs;
pub mod dependencies;
#[allow(unused_assignments)]
pub mod error;
mod gitignore;
mod init;
pub mod io;
pub mod license;
pub mod targets;

#[tokio::main]
async fn main() -> miette::Result<()> {
    let cli = Cli::parse();

    dotenvy::dotenv().ok();

    let mut config = read_main()?;

    if let Some(version) = cli.publish_version.clone() {
        config.package.version = version;
    }

    ensure_target_in_gitignore()?;

    let cli = Arc::new(cli);

    let profiles = read_profiles()?;

    if profiles.is_empty() {
        println!(
            "{} {}",
            style("[~]").yellow().bold(),
            style("No profiles configured in .buffy/").bold()
        );
        return Ok(());
    }

    let is_check = matches!(cli.command, Some(cli::Commands::Check));
    let publish = cli.publish;
    let operation = if is_check {
        "Check"
    } else if publish {
        "Publish"
    } else {
        "Build"
    };

    let total = profiles.len();
    println!(
        "{} {} {} profile(s)...",
        style("[-]").cyan().bold(),
        style(operation).bold(),
        style(total).bold()
    );

    let multi = MultiProgress::new();
    let mut tasks: JoinSet<(String, Result<()>)> = JoinSet::new();

    for profile in profiles {
        let pb = multi.add(ProgressBar::new_spinner());
        let package = config.package.clone();
        let source = config.source.clone();
        let profile_name = profile.name().to_string();

        tasks.spawn(async move {
            let result: Result<()> = async {
                let ctx = Context::new(package, source, profile, pb)?;
                if is_check {
                    check_profile_target(ctx.clone()).await?;
                    return Ok(());
                }
                build_profile_target(ctx.clone()).await?;
                if publish {
                    publish_profile_target(ctx).await?;
                }
                Ok(())
            }
            .await;
            (profile_name, result)
        });
    }

    let mut successes: Vec<String> = Vec::new();
    let mut errors: Vec<Error> = Vec::new();
    let mut failed_names: Vec<String> = Vec::new();

    while let Some(joined) = tasks.join_next().await {
        match joined {
            Ok((name, Ok(()))) => successes.push(name),
            Ok((name, Err(e))) => {
                failed_names.push(name);
                errors.push(e);
            }
            Err(join_err) => errors.push(Error::TaskPanicked {
                message: join_err.to_string(),
            }),
        }
    }

    multi.clear().unwrap();

    println!();
    if !successes.is_empty() {
        for name in &successes {
            println!(
                "  {} {} {}",
                style("✓").green().bold(),
                style(name).bold(),
                style(operation.to_lowercase() + " ok").dim()
            );
        }
    }
    if !failed_names.is_empty() {
        for name in &failed_names {
            println!(
                "  {} {} {}",
                style("✗").red().bold(),
                style(name).bold(),
                style("failed").dim()
            );
        }
    }

    println!();

    if !errors.is_empty() {
        println!(
            "{} {} {}",
            style("[!]").red().bold(),
            style(format!("{operation} failed")).bold(),
            style(format!(
                "({} succeeded, {} failed)",
                successes.len(),
                errors.len()
            ))
            .dim()
        );
        Err(AggregateError { errors }.into())
    } else {
        println!(
            "{} {} {}",
            style("[+]").green().bold(),
            style(format!("{operation} successful")).bold(),
            style(format!("({}/{})", successes.len(), total)).dim()
        );
        Ok(())
    }
}

#[derive(Debug, thiserror::Error, Diagnostic)]
#[error("{} profile(s) failed", errors.len())]
#[diagnostic(code(buffy::aggregate))]
struct AggregateError {
    #[related]
    errors: Vec<Error>,
}
