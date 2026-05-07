use std::path::PathBuf;

use semver::Version;

/// Generate and publish gRPC/Protobuf stubs for multiple languages
/// from a single Buffy.toml configuration file.
#[derive(Debug, clap::Parser)]
#[command(
    name = "buffy",
    version,
    about = "Generate and publish gRPC/Protobuf stubs for Go, Java, and Rust",
    long_about = "buffy reads your Buffy.toml and runs protoc with the correct \
                  plugins for each configured language, then publishes the \
                  generated packages to their respective registries."
)]
pub struct Cli {
    #[arg(short, long, default_value = "false")]
    pub publish: bool,

    #[arg(long)]
    pub publish_version: Option<Version>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Commands {
    Init {
        name: String,

        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    Check,
}
