//! What to launch: a `.dll` as is, or a `.csproj` built first and its
//! `TargetPath` read back. Shared by `bb debug` and the app's debugger.

use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::process::Command;

/// Why a target cannot be launched.
#[derive(Debug, thiserror::Error)]
pub enum TargetError {
    /// Not a `.csproj` or `.dll`.
    #[error("debug target must be a .csproj or .dll")]
    Kind,
    /// File or process error.
    #[error("debug I/O: {0}")]
    Io(#[from] std::io::Error),
    /// `dotnet build` (or the `TargetPath` query) failed.
    #[error("dotnet build failed: {0}")]
    Build(String),
    /// A `dotnet` command passed its deadline.
    #[error("dotnet command timed out")]
    Timeout,
}

/// The program to launch for `target`; a project is built first, each
/// `dotnet` call within `deadline`.
///
/// # Errors
/// [`TargetError`] on a wrong kind, a failed build or a timeout.
pub async fn resolve(target: &Path, deadline: Duration) -> Result<PathBuf, TargetError> {
    match target
        .extension()
        .and_then(|ext: &std::ffi::OsStr| ext.to_str())
    {
        Some("dll") => Ok(target.canonicalize()?),
        Some("csproj") => {
            let project: PathBuf = target.canonicalize()?;
            run(
                Command::new("dotnet")
                    .arg("build")
                    .arg(&project)
                    .arg("-v:q"),
                deadline,
            )
            .await?;
            let output: std::process::Output = run(
                Command::new("dotnet")
                    .arg("msbuild")
                    .arg(&project)
                    .arg("-getProperty:TargetPath"),
                deadline,
            )
            .await?;
            let path: PathBuf = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
            Ok(path.canonicalize()?)
        }
        _ => Err(TargetError::Kind),
    }
}

async fn run(
    command: &mut Command,
    deadline: Duration,
) -> Result<std::process::Output, TargetError> {
    let output: std::process::Output =
        tokio::time::timeout(deadline, command.kill_on_drop(true).output())
            .await
            .map_err(|_: tokio::time::error::Elapsed| TargetError::Timeout)??;
    if output.status.success() {
        Ok(output)
    } else {
        let stdout: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&output.stdout);
        let stderr: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&output.stderr);
        Err(TargetError::Build(
            format!("{stdout}{stderr}").trim().to_owned(),
        ))
    }
}
