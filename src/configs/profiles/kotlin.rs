use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Kotlin {
    #[serde(rename = "maven_central")]
    MavenCentral(MavenCentral),
    #[serde(rename = "git")]
    Git(Git),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MavenCentral {
    pub group_id: String,
    pub artifact_id: String,
    pub url: String,
    pub scm: Scm,
    pub protobuf_version: Option<String>,
    pub kotlin_version: Option<String>,

    /// If true, automatically publish after upload validates.
    /// Default: false (you confirm manually in the Sonatype portal).
    #[serde(default)]
    pub auto_publish: bool,

    /// What stage to wait for. "uploaded" returns quickly,
    /// "validated" waits for Sonatype validation,
    /// "published" waits for Maven Central indexing (can take 10-20 min).
    /// Default: "uploaded"
    #[serde(default = "default_wait_until")]
    pub wait_until: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Git {
    pub group_id: String,
    pub artifact_id: String,
    pub url: String,
    pub scm: Scm,
    pub remote: String,
    pub branch: String,
    pub protobuf_version: Option<String>,
    pub kotlin_version: Option<String>,
    #[serde(default)]
    pub keep: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scm {
    pub connection: String,
    pub url: String,
}

fn default_wait_until() -> String {
    "uploaded".to_string()
}
