use std::process::{Command, Stdio};

use console::style;
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

const DOCKERFILE: &str = include_str!("Dockerfile");
const IN_CONTAINER_MARKER: &str = "BUFFY_IN_CONTAINER";

fn image_tag() -> String {
    let mut hasher = Sha256::new();
    hasher.update(DOCKERFILE.as_bytes());
    let hash = hasher.finalize();
    let short: String = hash.iter().take(4).map(|b| format!("{b:02x}")).collect();
    format!("buffy-local:{}-{}", env!("CARGO_PKG_VERSION"), short)
}

pub fn is_in_container() -> bool {
    std::env::var(IN_CONTAINER_MARKER).is_ok()
}

fn detect_runtime() -> Result<&'static str> {
    if which::which("docker").is_ok() {
        Ok("docker")
    } else if which::which("podman").is_ok() {
        Ok("podman")
    } else {
        Err(Error::MissingProgram {
            program: "docker or podman".into(),
            hint: indoc::indoc! {"
                        Buffy's --container mode requires Docker or Podman:

                        • macOS:    brew install --cask docker
                        • Linux:    https://docs.docker.com/engine/install/
                                    or https://podman.io/docs/installation
                        • Windows:  https://docs.docker.com/desktop/install/windows-install/

                        Verify with: docker --version  (or: podman --version)
                    "}
            .into(),
        })
    }
}

fn image_exists(runtime: &str) -> bool {
    Command::new(runtime)
        .args(["image", "inspect", &image_tag()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn build_image(runtime: &str) -> Result<()> {
    eprintln!(
        "{} Building buffy container image (one-time, takes a while)...",
        style("[i]").cyan().bold(),
    );
    eprintln!(
        "    {}",
        style("Installs every language toolchain buffy supports. Cached after the first run.")
            .dim(),
    );

    let context = tempfile::tempdir()
        .map_err(|e| Error::Internal(format!("could not create build context: {e}")))?;

    let dockerfile_path = context.path().join("Dockerfile");
    std::fs::write(&dockerfile_path, DOCKERFILE)
        .map_err(|e| Error::Internal(format!("could not write Dockerfile: {e}")))?;

    let buffy_version = env!("CARGO_PKG_VERSION");

    let status = Command::new(runtime)
        .args([
            "build",
            "-t",
            &image_tag(),
            "--build-arg",
            &format!("BUFFY_VERSION={buffy_version}"),
            ".",
        ])
        .current_dir(context.path())
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| Error::Internal(format!("could not spawn {runtime}: {e}")))?;

    if !status.success() {
        return Err(Error::Internal(format!(
            "{runtime} build failed with exit code {}",
            status.code().unwrap_or(-1),
        )));
    }

    Ok(())
}

pub fn ensure_image(runtime: &str) -> Result<()> {
    if !image_exists(runtime) {
        build_image(runtime)?;
    }
    Ok(())
}

pub fn run_in_container() -> Result<std::process::ExitStatus> {
    let runtime = detect_runtime()?;
    ensure_image(runtime)?;

    let cwd = std::env::current_dir().map_err(|e| Error::Internal(format!("cwd: {e}")))?;

    let forwarded_args: Vec<String> = std::env::args()
        .skip(1)
        .filter(|a| a != "--container" && a != "-c")
        .collect();

    let mut cmd = Command::new(runtime);
    cmd.args(["run", "--rm", "-i"])
        .args(["-v", &format!("{}:/work", cwd.display())])
        .args(["-w", "/work"])
        .args(["-e", &format!("{IN_CONTAINER_MARKER}=1")]);

    // forward env vars
    const BLOCKED: &[&str] = &[
        "PATH",
        "HOME",
        "JAVA_HOME",
        "PWD",
        "OLDPWD",
        "SHELL",
        "USER",
        "LOGNAME",
        "HOSTNAME",
        "SHLVL",
        "_",
    ];
    for (key, _) in std::env::vars() {
        if BLOCKED.contains(&key.as_str()) || key == IN_CONTAINER_MARKER {
            continue;
        }
        cmd.args(["-e", &key]);
    }

    // SSH agent
    if let Ok(sock) = std::env::var("SSH_AUTH_SOCK") {
        cmd.args([
            "-v",
            &format!("{sock}:/ssh-agent"),
            "-e",
            "SSH_AUTH_SOCK=/ssh-agent",
        ]);
    }
    if let Some(home) = dirs::home_dir() {
        let known_hosts = home.join(".ssh").join("known_hosts");
        if known_hosts.exists() {
            cmd.args([
                "-v",
                &format!("{}:/root/.ssh/known_hosts:ro", known_hosts.display()),
            ]);
        }
    }

    if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        cmd.arg("-t");
    }

    cmd.arg(&image_tag()).args(&forwarded_args);
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    cmd.status()
        .map_err(|e| Error::Internal(format!("{runtime} run failed: {e}")))
}
