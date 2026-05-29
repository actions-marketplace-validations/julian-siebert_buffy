use std::sync::Arc;

use miette::{Diagnostic, NamedSource, SourceSpan};

use crate::dependencies::DependencyError;

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum Error {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Dependency(#[from] DependencyError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Config(#[from] crate::configs::Error),

    #[error(transparent)]
    #[diagnostic(transparent)]
    IO(#[from] crate::io::Error),

    #[error("Task panicked: {message}")]
    #[diagnostic(
        code(internal::task_panicked),
        help("This is a bug in buffy. Please report it.")
    )]
    TaskPanicked { message: String },

    #[error("Required environment variable `{name}` is not set")]
    #[diagnostic(code(env::missing), help("{hint}"))]
    MissingEnv { name: String, hint: String },

    #[error("Path error: {0}")]
    #[diagnostic(code(buffy::path), help("Is the program installed in PATH?"))]
    Which(#[from] which::Error),

    #[error("IO error: {0}")]
    #[diagnostic(code(buffy::io))]
    Io(#[from] Arc<std::io::Error>),

    #[error("Buffy.toml deserialization error: {message}")]
    #[diagnostic(code(buffy::config::parse))]
    BuffyTomlDeserialization {
        message: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("here")]
        span: Option<SourceSpan>,
    },

    #[error("Buffy.toml not found")]
    #[diagnostic(
        code(buffy::config::missing),
        help("Run `buffy init` to initialize a new Buffy project with `Buffy.toml`")
    )]
    BuffyTomlNotFound,

    #[error("Missing configuration: `{field}`")]
    #[diagnostic(code(buffy::config::missing_field))]
    MissingConfig {
        field: String,
        #[help]
        hint: String,
    },

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Prozess {cmd} beendet mit Code {code}")]
    #[diagnostic(code(buffy::command::failed))]
    CommandFailed {
        cmd: String,
        code: i32,
        #[help]
        help: Option<String>,
    },

    #[error("{count} compiler(s) failed")]
    #[diagnostic(code(buffy::build::failed), help("See the errors above for details."))]
    BuildFailed { count: usize },

    #[error("Required program `{program}` is not available")]
    #[diagnostic(code(buffy::missing_program), help("{hint}"))]
    MissingProgram { program: String, hint: String },

    #[error("Invalid SPDX expression: {expr}")]
    SpdxParse { expr: String },
}
pub type Result<T> = std::result::Result<T, Error>;

pub trait IoResultExt<T> {
    fn io_err(self) -> Result<T>;
}

impl<T> IoResultExt<T> for std::result::Result<T, std::io::Error> {
    fn io_err(self) -> Result<T> {
        self.map_err(|e| Error::Io(Arc::new(e)))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(Arc::new(e))
    }
}
