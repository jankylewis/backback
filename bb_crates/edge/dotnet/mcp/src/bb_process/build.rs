//! `dotnet build` of a server project; its output never reaches the protocol.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::time::timeout;

use crate::types::McpError;

const BUILD_DEADLINE: Duration = Duration::from_secs(300);
const BUILD_TAIL_LINES: usize = 20;

static BUILDS: Mutex<()> = Mutex::const_new(());

/// Build `project` and return its `TargetPath`.
pub(super) async fn build(project: &Path) -> Result<PathBuf, McpError> {
    let _one_build_at_a_time: tokio::sync::MutexGuard<'_, ()> = BUILDS.lock().await;
    let mut command: Command = Command::new("dotnet");
    command
        .args(["build", "-nologo", "-t:Build", "-getProperty:TargetPath"])
        .arg(project)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let child: Child = command
        .spawn()
        .map_err(|source: std::io::Error| McpError::Spawn {
            program: "dotnet".into(),
            source,
        })?;
    let output: std::process::Output = timeout(BUILD_DEADLINE, child.wait_with_output())
        .await
        .map_err(|_e: tokio::time::error::Elapsed| McpError::Timeout {
            operation: "dotnet build",
            seconds: BUILD_DEADLINE.as_secs(),
        })?
        .map_err(|source: std::io::Error| McpError::Io {
            operation: "dotnet build",
            source,
        })?;
    let stdout: String = String::from_utf8_lossy(&output.stdout).into_owned();
    let target: Option<PathBuf> = stdout
        .lines()
        .map(str::trim)
        .rfind(|line: &&str| !line.is_empty())
        .map(PathBuf::from)
        .filter(|path: &PathBuf| path.is_file());
    match target {
        Some(path) if output.status.success() => Ok(path),
        Some(_) | None => Err(McpError::Build {
            project: project.display().to_string(),
            status: output.status.to_string(),
            output: tail(&stdout, &String::from_utf8_lossy(&output.stderr)),
        }),
    }
}

fn tail(stdout: &str, stderr: &str) -> String {
    let lines: Vec<&str> = stdout
        .lines()
        .chain(stderr.lines())
        .filter(|line: &&str| !line.trim().is_empty())
        .collect();
    let skip: usize = lines.len().saturating_sub(BUILD_TAIL_LINES);
    lines
        .iter()
        .skip(skip)
        .copied()
        .collect::<Vec<&str>>()
        .join("\n")
}
