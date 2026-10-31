//! The official Roslyn language server (`roslyn-language-server`, MIT, `NuGet`).
//!
//! Beyond plain LSP it needs:
//! - flags `--stdio --logLevel <level> --extensionLogDirectory <dir>`;
//! - a `solution/open` (or `project/open`) notification after `initialized`;
//! - the client to wait for `workspace/projectInitializationComplete` before
//!   diagnostics and navigation are trustworthy.

use std::path::{Path, PathBuf};
use std::time::Duration;

use lsp_types::{
    ClientCapabilities, InitializeParams, InitializeResult, InitializedParams,
    WorkspaceClientCapabilities,
};
use serde_json::json;
use tokio::process::Command;
use url::Url;

use crate::bb_client::{LspClient, LspError, Settings};

mod bb_load;

pub use bb_load::{drain_log, log_incoming, wait_for_load};

/// Binary name of the official server (installed with `dotnet tool install -g roslyn-language-server --prerelease`).
pub const BINARY: &str = "roslyn-language-server";
/// Environment variable that overrides discovery.
pub const ENV_OVERRIDE: &str = "BACKBACK_ROSLYN_LS";
/// Notification the server sends once the solution is loaded.
pub const PROJECT_INIT_COMPLETE: &str = "workspace/projectInitializationComplete";

/// The setting that lets Roslyn run `dotnet restore` on the opened solution.
pub const AUTOMATIC_RESTORE: &str = "projects.dotnet_enable_automatic_restore";

/// MSBuild properties (read from the environment) that hold where a
/// design-time build writes: generated files (`obj/<config>/`) and outputs
/// (`bin/<config>/`), each under the scratch folder when restore is off.
pub const REDIRECTED_OUTPUTS: [(&str, &str); 2] =
    [("IntermediateOutputPath", "obj"), ("OutputPath", "bin")];

/// Whether Roslyn may write into the opened solution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Restore {
    /// The default: no `dotnet restore`, and the design-time build writes its
    /// generated files to a scratch folder outside the solution.
    Off,
    /// Let Roslyn restore projects it finds unrestored and build in `obj/`.
    On,
}

/// The `workspace/configuration` answers bb gives Roslyn.
#[must_use]
pub fn settings(restore: Restore) -> Settings {
    let allowed: bool = match restore {
        Restore::Off => false,
        Restore::On => true,
    };
    let mut map: Settings = Settings::new();
    map.insert(
        AUTOMATIC_RESTORE.to_owned(),
        serde_json::Value::Bool(allowed),
    );
    map
}

/// Roslyn-specific errors.
#[derive(Debug, thiserror::Error)]
pub enum RoslynError {
    /// The server binary was not found.
    #[error("{BINARY} not found; set {ENV_OVERRIDE} or run: dotnet tool install -g {BINARY} --prerelease")]
    NotFound,
    /// A path could not be turned into a URI.
    #[error("path is not absolute or not valid as a URI: {0}")]
    BadPath(PathBuf),
    /// The solution did not finish loading within the deadline.
    #[error("solution did not finish loading within {after:?}: {loaded} of {expected} projects")]
    LoadTimeout {
        /// The deadline.
        after: Duration,
        /// Projects the server reported loaded.
        loaded: usize,
        /// Projects the solution declares.
        expected: usize,
    },
    /// The notification channel was already taken from the client.
    #[error("incoming channel already taken")]
    NoIncoming,
    /// The run log could not be written.
    #[error("cannot write the server log: {0}")]
    Log(#[source] std::io::Error),
    /// Underlying client error.
    #[error(transparent)]
    Client(#[from] LspError),
}

/// Find the server: env override, then `PATH`, then `~/.dotnet/tools`.
///
/// # Errors
/// [`RoslynError::NotFound`] when no binary exists at any of those places.
pub fn locate() -> Result<PathBuf, RoslynError> {
    if let Some(p) = std::env::var_os(ENV_OVERRIDE) {
        let p: PathBuf = PathBuf::from(p);
        return if p.is_file() {
            Ok(p)
        } else {
            Err(RoslynError::NotFound)
        };
    }
    let names: [String; 2] = [BINARY.to_owned(), format!("{BINARY}.exe")];
    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p: std::ffi::OsString| std::env::split_paths(&p).collect())
        .unwrap_or_default();
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        dirs.push(PathBuf::from(home).join(".dotnet").join("tools"));
    }
    dirs.iter()
        .flat_map(|d: &PathBuf| names.iter().map(move |n: &String| d.join(n)))
        .find(|p: &PathBuf| p.is_file())
        .ok_or(RoslynError::NotFound)
}

/// Start the server over stdio. `log_dir` receives the server's own logs;
/// with [`Restore::Off`], `scratch` receives the design-time build output.
///
/// # Errors
/// A folder cannot be created, or the process cannot be spawned.
pub fn start(
    binary: &Path,
    log_dir: &Path,
    timeout: Duration,
    restore: Restore,
    scratch: &Path,
) -> Result<LspClient, RoslynError> {
    create_dir(log_dir)?;
    let mut cmd: Command = Command::new(binary);
    match restore {
        Restore::Off => {
            for (property, folder) in REDIRECTED_OUTPUTS {
                let dir: PathBuf = scratch.join(folder);
                create_dir(&dir)?;
                // MSBuild needs the trailing separator on an output path.
                let mut value: std::ffi::OsString = dir.into_os_string();
                value.push(std::path::MAIN_SEPARATOR_STR);
                cmd.env(property, value);
            }
        }
        Restore::On => {
            for (property, _) in REDIRECTED_OUTPUTS {
                cmd.env_remove(property);
            }
        }
    }
    cmd.arg("--stdio")
        .arg("--logLevel")
        .arg("Information")
        .arg("--extensionLogDirectory")
        .arg(log_dir);
    Ok(LspClient::spawn(cmd, timeout, settings(restore))?)
}

fn create_dir(dir: &Path) -> Result<(), RoslynError> {
    std::fs::create_dir_all(dir)
        .map_err(|_: std::io::Error| RoslynError::BadPath(dir.to_path_buf()))
}

/// `initialize` + `initialized` for a workspace root.
///
/// # Errors
/// A bad root path, or the `initialize` request fails.
pub async fn initialize(client: &LspClient, root: &Path) -> Result<InitializeResult, RoslynError> {
    let root_uri: Url = file_url(root)?;
    #[allow(deprecated)] // root_uri is deprecated in LSP but Roslyn still reads it
    let params: InitializeParams = InitializeParams {
        process_id: Some(std::process::id()),
        root_uri: Some(root_uri),
        capabilities: ClientCapabilities {
            workspace: Some(WorkspaceClientCapabilities {
                configuration: Some(true),
                ..WorkspaceClientCapabilities::default()
            }),
            ..ClientCapabilities::default()
        },
        ..InitializeParams::default()
    };
    let result: InitializeResult = client.request("initialize", params).await?;
    client.notify("initialized", InitializedParams {}).await?;
    Ok(result)
}

/// Tell the server which solution to load.
///
/// # Errors
/// A bad solution path, or the connection is closed.
pub async fn open_solution(client: &LspClient, solution: &Path) -> Result<(), RoslynError> {
    let uri: Url = file_url(solution)?;
    client
        .notify("solution/open", json!({ "solution": uri }))
        .await?;
    Ok(())
}

fn file_url(path: &Path) -> Result<Url, RoslynError> {
    Url::from_file_path(path).map_err(|(): ()| RoslynError::BadPath(path.to_path_buf()))
}

#[cfg(test)]
mod tests;
