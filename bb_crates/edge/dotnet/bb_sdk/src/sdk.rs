//! Locate the `dotnet` host and read the SDK version. Every process call has a deadline.

use std::path::PathBuf;
use std::process::Output;
use std::time::Duration;

use tokio::process::Command;

/// SDK errors.
#[derive(Debug, thiserror::Error)]
pub enum SdkError {
    /// No `dotnet` on `PATH` or in `DOTNET_ROOT`.
    #[error("dotnet not found on PATH or DOTNET_ROOT")]
    NotFound,
    /// The process failed to start or exited non-zero.
    #[error("dotnet {args}: {detail}")]
    Process {
        /// Arguments that were run.
        args: String,
        /// What went wrong.
        detail: String,
    },
    /// The deadline passed.
    #[error("dotnet {args} timed out after {after:?}")]
    Timeout {
        /// Arguments that were run.
        args: String,
        /// The deadline.
        after: Duration,
    },
}

/// The SDK version string as `dotnet --version` prints it, e.g. `10.0.403`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdkVersion(String);

impl SdkVersion {
    /// The raw version text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A located .NET host.
#[derive(Debug, Clone)]
pub struct Sdk {
    host: PathBuf,
}

impl Sdk {
    /// Find `dotnet`: `DOTNET_ROOT` first, then `PATH`.
    ///
    /// # Errors
    /// [`SdkError::NotFound`] when neither place has it.
    pub fn locate() -> Result<Self, SdkError> {
        let names: [&str; 2] = ["dotnet", "dotnet.exe"];
        let mut dirs: Vec<PathBuf> = Vec::new();
        if let Some(root) = std::env::var_os("DOTNET_ROOT") {
            dirs.push(PathBuf::from(root));
        }
        if let Some(path) = std::env::var_os("PATH") {
            dirs.extend(std::env::split_paths(&path));
        }
        dirs.iter()
            .flat_map(|d: &PathBuf| names.iter().map(move |n: &&str| d.join(n)))
            .find(|p: &PathBuf| p.is_file())
            .map(|host: PathBuf| Self { host })
            .ok_or(SdkError::NotFound)
    }

    /// Path of the `dotnet` host.
    #[must_use]
    pub fn host(&self) -> &PathBuf {
        &self.host
    }

    /// `dotnet --version`, bounded by `timeout`.
    ///
    /// # Errors
    /// See [`Sdk::run`].
    pub async fn version(&self, timeout: Duration) -> Result<SdkVersion, SdkError> {
        let out: String = self.run(&["--version"], timeout).await?;
        Ok(SdkVersion(out.trim().to_owned()))
    }

    /// Run `dotnet <args>` and return stdout. Non-zero exit is an error.
    ///
    /// # Errors
    /// Start failure, non-zero exit, or the deadline passing.
    pub async fn run(&self, args: &[&str], timeout: Duration) -> Result<String, SdkError> {
        let shown: String = args.join(" ");
        let mut cmd: Command = Command::new(&self.host);
        cmd.args(args).kill_on_drop(true);
        let late: SdkError = SdkError::Timeout {
            args: shown.clone(),
            after: timeout,
        };
        let output: Output = tokio::time::timeout(timeout, cmd.output())
            .await
            .map_err(|_: tokio::time::error::Elapsed| late)?
            .map_err(|e: std::io::Error| SdkError::Process {
                args: shown.clone(),
                detail: e.to_string(),
            })?;
        if !output.status.success() {
            return Err(SdkError::Process {
                args: shown,
                detail: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}
