use std::path::PathBuf;

use miette::Diagnostic;

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum Error {
    #[error("Permission denied for {path}")]
    #[diagnostic(
        code(fs::permission_denied),
        help("Check the file permissions with `ls -l {}`.", path.display())
    )]
    PermissionDenied {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("File not found: {path}")]
    #[diagnostic(
        code(fs::not_found),
        help("Make sure the file exists and the path is correct.")
    )]
    NotFound {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Expected {path} to be a directory, but it is a file")]
    #[diagnostic(
        code(fs::not_a_directory),
        help("Remove the file at {} or rename it, then create a directory there.", path.display())
    )]
    NotADirectory { path: PathBuf },

    #[error("I/O error at {path}")]
    #[diagnostic(code(fs::io))]
    Other {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to spawn `{program}`")]
    #[diagnostic(
        code(cmd::spawn_failed),
        help("Make sure `{program}` is installed and on your PATH.")
    )]
    CommandSpawn {
        program: String,
        #[source]
        source: std::io::Error,
    },

    #[error("`{program}` exited with status {code}")]
    #[diagnostic(
        code(cmd::failed),
        help("Full command: {program} {}\n\nCheck the output above for details from `{program}`.", args.join(" "))
    )]
    CommandFailed {
        program: String,
        args: Vec<String>,
        code: i32,
    },
}
