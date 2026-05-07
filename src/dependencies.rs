use std::path::PathBuf;

use miette::Diagnostic;
use which::which;

#[derive(Debug, Clone, thiserror::Error, Diagnostic)]
pub enum DependencyError {
    #[error("`git` not found in PATH")]
    #[diagnostic(
        code(deps::git),
        help(
            "Install Git:\n\
                 \n\
                 • macOS:    brew install git  (or use Xcode Command Line Tools)\n\
                 • Debian:   apt install git\n\
                 • Arch:     pacman -S git\n\
                 • Windows:  scoop install git  (or download from https://git-scm.com/download/win)\n\
                 \n\
                 After installing, verify with: git --version"
        )
    )]
    Git,

    #[error("`protoc` not found in PATH")]
    #[diagnostic(
        code(deps::protoc),
        help(
            "Install the Protocol Buffers compiler:\n\
                \n\
                • macOS:    brew install protobuf\n\
                • Debian:   apt install protobuf-compiler\n\
                • Arch:     pacman -S protobuf\n\
                • Windows:  scoop install protobuf  (or download from https://github.com/protocolbuffers/protobuf/releases)\n\
                \n\
                After installing, verify with: protoc --version"
        )
    )]
    Protoc,

    #[error("`protoc-gen-go` not found in PATH")]
    #[diagnostic(
        code(deps::protoc_gen_go),
        help(
            "Install the Go protobuf plugin:\n\
                 \n\
                 go install google.golang.org/protobuf/cmd/protoc-gen-go@latest\n\
                 \n\
                 Make sure $(go env GOPATH)/bin is in your PATH."
        )
    )]
    ProtocGenGo,

    #[error("`protoc-gen-go-grpc` not found in PATH")]
    #[diagnostic(
        code(deps::protoc_gen_go_grpc),
        help(
            "Install the Go gRPC plugin:\n\
                 \n\
                 go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest\n\
                 \n\
                 Make sure $(go env GOPATH)/bin is in your PATH."
        )
    )]
    ProtocGenGoGrpc,

    #[error("`go` not found in PATH")]
    #[diagnostic(
        code(deps::go),
        help(
            "Install Go:\n\
                 \n\
                 • macOS:    brew install go\n\
                 • Debian:   apt install golang-go\n\
                 • Arch:     pacman -S go\n\
                 • Windows:  scoop install go  (or download from https://go.dev/dl/)\n\
                 \n\
                 After installing, verify with: go version"
        )
    )]
    Go,

    #[error("`java` not found in PATH")]
    #[diagnostic(
        code(deps::java),
        help(
            "Install a JDK (17 or newer recommended):\n\
                 \n\
                 • macOS:    brew install openjdk\n\
                 • Debian:   apt install default-jdk\n\
                 • Arch:     pacman -S jdk-openjdk\n\
                 • Windows:  scoop install openjdk\n\
                 \n\
                 After installing, verify with: java --version"
        )
    )]
    Java,

    #[error("`mvn` not found in PATH")]
    #[diagnostic(
        code(deps::maven),
        help(
            "Install Apache Maven:\n\
                 \n\
                 • macOS:    brew install maven\n\
                 • Debian:   apt install maven\n\
                 • Arch:     pacman -S maven\n\
                 • Windows:  scoop install maven  (or download from https://maven.apache.org/download.cgi)\n\
                 \n\
                 After installing, verify with: mvn --version"
        )
    )]
    Maven,

    #[error("`cargo` not found in PATH")]
    #[diagnostic(
        code(deps::cargo),
        help(
            "Install the Rust toolchain via rustup:\n\
                 \n\
                 curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh\n\
                 \n\
                 After installing, restart your shell and verify with: cargo --version"
        )
    )]
    Cargo,
}

pub fn git() -> Result<PathBuf, DependencyError> {
    which("git").map_err(|_| DependencyError::Git)
}

#[macro_export]
macro_rules! git {
    ($ctx:expr, env: [$(($k:expr, $v:expr)),* $(,)?], $($arg:expr),+ $(,)?) => {{
        let ctx = &$ctx;
        let mut cmd = ::tokio::process::Command::new("git");
        cmd.args([$($arg),+]).current_dir(&ctx.target_path);
        $( cmd.env($k, $v); )*
        ctx.run(&mut cmd).await
    }};

    ($ctx:expr, $($arg:expr),+ $(,)?) => {{
        let ctx = &$ctx;
        let mut cmd = ::tokio::process::Command::new("git");
        cmd.args([$($arg),+]).current_dir(&ctx.target_path);
        ctx.run(&mut cmd).await
    }};
}

pub fn protoc() -> Result<PathBuf, DependencyError> {
    which("protoc").map_err(|_| DependencyError::Protoc)
}

pub fn protoc_gen_go() -> Result<PathBuf, DependencyError> {
    which("protoc-gen-go").map_err(|_| DependencyError::ProtocGenGo)
}

pub fn protoc_gen_go_grpc() -> Result<PathBuf, DependencyError> {
    which("protoc-gen-go-grpc").map_err(|_| DependencyError::ProtocGenGoGrpc)
}

pub fn go() -> Result<PathBuf, DependencyError> {
    which("go").map_err(|_| DependencyError::Go)
}

pub fn java() -> Result<PathBuf, DependencyError> {
    which("java").map_err(|_| DependencyError::Java)
}

pub fn maven() -> Result<PathBuf, DependencyError> {
    which("mvn").map_err(|_| DependencyError::Maven)
}

pub fn cargo() -> Result<PathBuf, DependencyError> {
    which("cargo").map_err(|_| DependencyError::Cargo)
}
