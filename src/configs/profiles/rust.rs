use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Rust {
    #[serde(rename = "crate")]
    Crate(Crate),
    #[serde(rename = "git")]
    Git(Git),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Crate {
    pub name: Option<String>,
    pub edition: String,
    pub repository: String,
    pub documentation: String,
    /// "crates-io" for the public registry, or a custom registry name
    /// configured in ~/.cargo/config.toml
    #[serde(default = "default_registry")]
    pub registry: String,
    pub prost_version: Option<String>,
    pub tonic_version: Option<String>,
    #[serde(default = "default_grpc")]
    pub grpc: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Git {
    pub name: Option<String>,
    pub edition: String,
    pub remote: String,
    pub branch: String,
    pub repository: String,
    pub documentation: String,
    pub prost_version: Option<String>,
    pub tonic_version: Option<String>,
    #[serde(default = "default_grpc")]
    pub grpc: bool,
    #[serde(default)]
    pub keep: Vec<String>,
}

fn default_registry() -> String {
    "crates-io".to_string()
}

fn default_grpc() -> bool {
    true
}
