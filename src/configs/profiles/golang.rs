use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Golang {
    #[serde(rename = "git")]
    Git(Git),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Git {
    pub module: String,
    pub remote: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    #[serde(default = "default_keep")]
    pub keep: Vec<String>,

    #[serde(default = "default_grpc")]
    pub grpc: bool,
}

fn default_branch() -> String {
    "main".into()
}

fn default_keep() -> Vec<String> {
    vec![]
}

fn default_grpc() -> bool {
    true
}
