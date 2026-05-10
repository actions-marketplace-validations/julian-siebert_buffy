use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeScript {
    #[serde(rename = "npm")]
    Npm(Npm),
    #[serde(rename = "git")]
    Git(Git),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Npm {
    pub name: String,
    /// e.g. "https://registry.npmjs.org/" or "https://npm.pkg.github.com/"
    #[serde(default = "default_registry")]
    pub registry: String,
    /// "public" or "restricted" — only relevant for scoped packages
    #[serde(default = "default_access")]
    pub access: String,
    pub repository: String,
    pub homepage: Option<String>,
    #[serde(default = "default_grpc")]
    pub grpc: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Git {
    pub name: String,
    pub remote: String,
    pub branch: String,
    pub repository: String,
    pub homepage: Option<String>,
    #[serde(default = "default_grpc")]
    pub grpc: bool,
    #[serde(default)]
    pub keep: Vec<String>,
}

fn default_registry() -> String {
    "https://registry.npmjs.org/".to_string()
}

fn default_access() -> String {
    "public".to_string()
}

fn default_grpc() -> bool {
    true
}
