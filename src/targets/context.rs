use std::{ops::Deref, path::PathBuf, process::Stdio, sync::Arc, time::Duration};

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

use crate::{
    configs::{Package, Source, profiles::NamedProfile},
    error::Result,
};

pub const TARGETS_DIRECTORY_PATH: &str = "target";

#[derive(Clone)]
pub struct Context(Arc<ContextInner>);

pub struct ContextInner {
    pub package: Package,
    pub source: Source,
    pub profile: NamedProfile,
    pub target_path: PathBuf,
    pub pb: ProgressBar,
    proto_files: Vec<PathBuf>,
}

impl Context {
    pub fn new(
        package: Package,
        source: Source,
        profile: NamedProfile,
        pb: ProgressBar,
    ) -> Result<Self> {
        let target_path = PathBuf::from(TARGETS_DIRECTORY_PATH).join(profile.name());

        if crate::io::exists(&target_path)? {
            crate::io::remove_dir_all(&target_path)?;
        }

        crate::io::create_dir_all(&target_path)?;

        pb.set_style(
            ProgressStyle::with_template("{spinner:.cyan} {prefix:.bold} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );

        pb.set_prefix(format!("[{}]", profile.name()));
        pb.enable_steady_tick(Duration::from_millis(80));

        let proto_files = collect_proto_files(&source.path)?;

        Ok(Self(Arc::new(ContextInner {
            package,
            source,
            profile,
            target_path,
            pb,
            proto_files,
        })))
    }

    pub async fn run(&self, cmd: &mut Command) -> Result<()> {
        let std_cmd = cmd.as_std();
        let program = std_cmd.get_program().to_string_lossy().into_owned();

        let args: Vec<String> = std_cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();

        let mut child = cmd
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|source| crate::io::Error::CommandSpawn {
                program: program.clone(),
                source,
            })?;

        let stderr = child.stderr.take().unwrap();

        let pb = self.pb.clone();
        let prefix = self.profile.name().to_string();
        let cmd_display = program.clone();

        let err_task = tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                pb.suspend(|| {
                    eprintln!(
                        "{} {} {} {line}",
                        style("[~]").yellow().bold(),
                        style(prefix.to_uppercase()).bold(),
                        style(&cmd_display).cyan()
                    )
                })
            }
        });

        let (_, status) = tokio::join!(err_task, child.wait());
        let status = status.map_err(|source| crate::io::Error::CommandSpawn {
            program: program.clone(),
            source,
        })?;

        if !status.success() {
            return Err(crate::io::Error::CommandFailed {
                program,
                args,
                code: status.code().unwrap_or(-1),
            }
            .into());
        }

        Ok(())
    }

    pub fn proto_files(&self) -> &[PathBuf] {
        &self.0.proto_files
    }

    pub fn finish_check(&self) {
        self.pb
            .finish_with_message(format!("check ok for `{}`", self.profile.name()));
    }

    pub fn finish_build(&self) {
        self.pb
            .finish_with_message(format!("built `{}`", self.profile.name()));
    }

    pub fn finish_publish(&self, tag: &str, remote: &str) {
        self.pb.finish_with_message(format!(
            "published {tag} → {remote} for `{}`",
            self.profile.name()
        ));
    }
}

fn collect_proto_files(root: &std::path::Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    collect_recursive(root, &mut out)?;
    Ok(out)
}

fn collect_recursive(dir: &std::path::Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in crate::io::read_dir(dir)? {
        let path = entry?;
        if path.is_dir() {
            collect_recursive(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("proto") {
            out.push(path);
        }
    }
    Ok(())
}

impl Deref for Context {
    type Target = ContextInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
