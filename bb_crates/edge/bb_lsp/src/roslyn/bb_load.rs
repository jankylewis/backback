//! Waiting for the solution load, and the run log it leaves behind.

use std::time::{Duration, Instant};

use bb_dotnet::ProjectPath;

use super::{LoadState, RoslynError, PROJECT_INIT_COMPLETE};
use crate::bb_client::{Incoming, LspError};

/// How a finished load ended: how long it took, and the projects Roslyn
/// reported it could not load (each one logged its reason).
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct Loaded {
    /// From the call until the last project settled.
    pub took: Duration,
    /// Projects that failed to load and never loaded after.
    pub failed: Vec<ProjectPath>,
}

/// Drain incoming messages until the solution is loaded: the server has sent
/// `projectInitializationComplete` and every distinct project of `projects`
/// has a load or a failure line. Returns how long it took and what failed.
///
/// The signal alone is not enough: Roslyn sends it after the first batch
/// (observed: 1 of 27 projects on `Apex`, 1 of 17 on `MbTx`) while the rest are
/// still loading. Batch counts are not enough either: a batch can reload a
/// project already loaded (bus 066), so readiness is counted by path.
///
/// Every `window/logMessage` and `window/showMessage` on the way is written to
/// `log` as one line, so the run leaves the server's own account behind.
///
/// # Errors
/// [`RoslynError::LoadTimeout`] past `deadline` (naming the projects still
/// missing), the server closes, or `log` cannot be written.
pub async fn wait_for_load<W: std::io::Write>(
    incoming: &mut tokio::sync::mpsc::Receiver<Incoming>,
    deadline: Duration,
    projects: &[ProjectPath],
    log: &mut W,
) -> Result<Loaded, RoslynError> {
    let started: Instant = Instant::now();
    let mut signalled: bool = false;
    let mut state: LoadState = LoadState::new(projects);
    while !(signalled && state.settled()) {
        let Some(remaining): Option<Duration> = deadline.checked_sub(started.elapsed()) else {
            return Err(RoslynError::timeout(deadline, &state));
        };
        let Ok(next): Result<Option<Incoming>, tokio::time::error::Elapsed> =
            tokio::time::timeout(remaining, incoming.recv()).await
        else {
            return Err(RoslynError::timeout(deadline, &state));
        };
        match next {
            Some(Incoming::Notification { method, .. }) if method == PROJECT_INIT_COMPLETE => {
                signalled = true;
            }
            Some(message) => {
                let text: &str = log_incoming(&message, log)?;
                // Whether it changed the state matters only to a progress view.
                let _changed: bool = state.observe(text);
            }
            None => return Err(RoslynError::Client(LspError::Closed)),
        }
    }
    Ok(Loaded {
        took: started.elapsed(),
        failed: state.failed(),
    })
}

/// Write one server message to `log` the way the run log keeps it: a
/// `window/logMessage` or `window/showMessage` as `[level] text`, each
/// section a `workspace/configuration` request asks for as `[config] name`.
/// Returns the message text (empty for anything else). The session calls
/// this for every message after the load too, so the log never stops there.
///
/// # Errors
/// [`RoslynError::Log`] when `log` cannot be written.
pub fn log_incoming<'m, W: std::io::Write>(
    message: &'m Incoming,
    log: &mut W,
) -> Result<&'m str, RoslynError> {
    match message {
        Incoming::Notification { method, params } => log_message(method, params.as_ref(), log),
        Incoming::ServerRequest { method, params } => {
            log_configuration(method, params.as_ref(), log)?;
            Ok("")
        }
    }
}

/// Write every message still coming to `log` until the server closes the
/// stream or `deadline` passes. Call it after `shutdown`, so the lines the
/// server writes on its way out stay in the run log.
///
/// # Errors
/// [`RoslynError::Log`] when `log` cannot be written. A passed deadline is
/// not an error: the log then simply ends there.
pub async fn drain_log<W: std::io::Write>(
    incoming: &mut tokio::sync::mpsc::Receiver<Incoming>,
    deadline: Duration,
    log: &mut W,
) -> Result<(), RoslynError> {
    let started: Instant = Instant::now();
    while let Some(remaining) = deadline.checked_sub(started.elapsed()) {
        match tokio::time::timeout(remaining, incoming.recv()).await {
            Ok(Some(message)) => {
                log_incoming(&message, log)?;
            }
            Ok(None) | Err(_) => break,
        }
    }
    Ok(())
}

/// Lines Roslyn writes when a restore it started has ended, either way.
const RESTORE_ENDS: [&str; 3] = ["Restore complete", "restore was canceled", "restore failed"];

/// With restore on, Roslyn starts `dotnet restore` after the load, so a run
/// that stops at the load can exit first (BB-LSP-004). Log every message
/// until a restore end line; `true` when one came, `false` when `deadline`
/// passed or the server closed first.
///
/// # Errors
/// [`RoslynError::Log`] when `log` cannot be written.
pub async fn wait_for_restore<W: std::io::Write>(
    incoming: &mut tokio::sync::mpsc::Receiver<Incoming>,
    deadline: Duration,
    log: &mut W,
) -> Result<bool, RoslynError> {
    let started: Instant = Instant::now();
    while let Some(remaining) = deadline.checked_sub(started.elapsed()) {
        match tokio::time::timeout(remaining, incoming.recv()).await {
            Ok(Some(message)) => {
                let text: &str = log_incoming(&message, log)?;
                if RESTORE_ENDS.iter().any(|end: &&str| text.contains(end)) {
                    return Ok(true);
                }
            }
            Ok(None) | Err(_) => break,
        }
    }
    Ok(false)
}

/// Write each section a `workspace/configuration` request asked for.
fn log_configuration<W: std::io::Write>(
    method: &str,
    params: Option<&serde_json::Value>,
    log: &mut W,
) -> Result<(), RoslynError> {
    if method != "workspace/configuration" {
        return Ok(());
    }
    let items: &[serde_json::Value] = params
        .and_then(|p: &serde_json::Value| p.get("items"))
        .and_then(serde_json::Value::as_array)
        .map_or(&[], Vec::as_slice);
    for item in items {
        let section: &str = item
            .get("section")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        writeln!(log, "[config] {section}").map_err(RoslynError::Log)?;
    }
    Ok(())
}

/// Write one `window/logMessage` or `window/showMessage` as `[level] text`;
/// returns the text (empty for any other method).
fn log_message<'p, W: std::io::Write>(
    method: &str,
    params: Option<&'p serde_json::Value>,
    log: &mut W,
) -> Result<&'p str, RoslynError> {
    if method != "window/logMessage" && method != "window/showMessage" {
        return Ok("");
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
    writeln!(log, "[{level}] {text}").map_err(RoslynError::Log)?;
    Ok(text)
}
