use std::path::PathBuf;

use semver::Version;
use serde::{Deserialize, Serialize};

use crate::configs::author::Author;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Main {
    pub package: Package,
    #[serde(default)]
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub description: String,
    pub version: Version,
    pub license: String,
    pub authors: Vec<Author>,
    pub homepage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub path: PathBuf,
}

impl Default for Source {
    fn default() -> Self {
        Self { path: "src".into() }
    }
}
