//! The official Roslyn language server (`roslyn-language-server`, MIT, `NuGet`).
//!
//! Beyond plain LSP it needs:
//! - flags `--stdio --logLevel <level> --extensionLogDirectory <dir>`;
//! - a `solution/open` (or `project/open`) notification after `initialized`;
//! - the client to wait for `workspace/projectInitializationComplete` before
//!   diagnostics and navigation are trustworthy.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use lsp_types::{ClientCapabilities, InitializeParams, InitializeResult, InitializedParams};
use serde_json::json;
use tokio::process::Command;
use url::Url;

use crate::client::{Incoming, LspClient, LspError};

/// Binary name of the official server (installed with `dotnet tool install -g roslyn-language-server --prerelease`).
pub const BINARY: &str = "roslyn-language-server";
/// Environment variable that overrides discovery.
pub const ENV_OVERRIDE: &str = "BACKBACK_ROSLYN_LS";
/// Notification the server sends once the solution is loaded.
pub const PROJECT_INIT_COMPLETE: &str = "workspace/projectInitializationComplete";

/// Roslyn-specific errors.
#[derive(Debug, thiserror::Error)]
pub enum RoslynError {
    /// The server binary was not found.
    #[error("{BINARY} not found; set {ENV_OVERRIDE} or run: dotnet tool install -g {BINARY} --prerelease")]
    NotFound,
    /// A path could not be turned into a URI.
    #[error("path is not absolute or not valid as a URI: {0}")]
    BadPath(PathBuf),
    /// The server did not report initialization within the deadline.
    #[error("solution did not finish loading within {0:?}")]
    LoadTimeout(Duration),
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

/// Start the server over stdio. `log_dir` receives the server's own logs.
///
/// # Errors
/// The log folder cannot be created, or the process cannot be spawned.
pub fn start(binary: &Path, log_dir: &Path, timeout: Duration) -> Result<LspClient, RoslynError> {
    std::fs::create_dir_all(log_dir)
        .map_err(|_: std::io::Error| RoslynError::BadPath(log_dir.to_path_buf()))?;
    let mut cmd: Command = Command::new(binary);
    cmd.arg("--stdio")
        .arg("--logLevel")
        .arg("Information")
        .arg("--extensionLogDirectory")
        .arg(log_dir);
    Ok(LspClient::spawn(cmd, timeout)?)
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
        capabilities: ClientCapabilities::default(),
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

/// Drain incoming messages until `projectInitializationComplete`. Returns how long it took.
///
/// Every `window/logMessage` and `window/showMessage` on the way is written to
/// `log` as one line, so the run leaves the server's own account behind.
///
/// # Errors
/// [`RoslynError::LoadTimeout`] past `deadline`, the server closes, or `log`
/// cannot be written.
pub async fn wait_for_load<W: std::io::Write>(
    incoming: &mut tokio::sync::mpsc::Receiver<Incoming>,
    deadline: Duration,
    log: &mut W,
) -> Result<Duration, RoslynError> {
    let started: Instant = Instant::now();
    loop {
        let remaining: Duration = deadline
            .checked_sub(started.elapsed())
            .ok_or(RoslynError::LoadTimeout(deadline))?;
        let next: Option<Incoming> = tokio::time::timeout(remaining, incoming.recv())
            .await
            .map_err(|_: tokio::time::error::Elapsed| RoslynError::LoadTimeout(deadline))?;
        match next {
            Some(Incoming::Notification { method, .. }) if method == PROJECT_INIT_COMPLETE => {
                return Ok(started.elapsed());
            }
            Some(Incoming::Notification { method, params }) => {
                log_message(&method, params.as_ref(), log)?;
            }
            Some(Incoming::ServerRequest { .. }) => {}
            None => return Err(RoslynError::Client(LspError::Closed)),
        }
    }
}

/// Write one `window/logMessage` or `window/showMessage` as `[level] text`.
fn log_message<W: std::io::Write>(
    method: &str,
    params: Option<&serde_json::Value>,
    log: &mut W,
) -> Result<(), RoslynError> {
    if method != "window/logMessage" && method != "window/showMessage" {
        return Ok(());
    }
    let text: &str = params
        .and_then(|p: &serde_json::Value| p.get("message"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let level: &str = match params
        .and_then(|p: &serde_json::Value| p.get("type"))
        .and_then(serde_json::Value::as_u64)
    {
        Some(1) => "error",
        Some(2) => "warn",
        Some(3) => "info",
        Some(_) | None => "log",
    };
    writeln!(log, "[{level}] {text}").map_err(RoslynError::Log)
}

fn file_url(path: &Path) -> Result<Url, RoslynError> {
    Url::from_file_path(path).map_err(|(): ()| RoslynError::BadPath(path.to_path_buf()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    fn note(method: &str, params: serde_json::Value) -> Incoming {
        Incoming::Notification {
            method: method.to_owned(),
            params: Some(params),
        }
    }

    #[tokio::test]
    async fn wait_for_load_keeps_every_server_message_in_the_log() {
        let (tx, mut rx): (mpsc::Sender<Incoming>, mpsc::Receiver<Incoming>) = mpsc::channel(8);
        tx.send(note(
            "window/logMessage",
            json!({"type": 3, "message": "loading A.csproj"}),
        ))
        .await
        .unwrap();
        tx.send(note(
            "window/showMessage",
            json!({"type": 1, "message": "restore failed"}),
        ))
        .await
        .unwrap();
        tx.send(note("$/progress", json!({"token": 1})))
            .await
            .unwrap();
        tx.send(note(PROJECT_INIT_COMPLETE, json!({})))
            .await
            .unwrap();
        let mut log: Vec<u8> = Vec::new();
        wait_for_load(&mut rx, Duration::from_secs(5), &mut log)
            .await
            .unwrap();
        let text: String = String::from_utf8(log).unwrap();
        assert_eq!(text, "[info] loading A.csproj\n[error] restore failed\n");
    }
}
