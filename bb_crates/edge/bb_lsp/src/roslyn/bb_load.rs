//! Waiting for the solution load, and the run log it leaves behind.

use std::time::{Duration, Instant};

use super::{RoslynError, PROJECT_INIT_COMPLETE};
use crate::bb_client::{Incoming, LspError};

/// The log line Roslyn writes after each batch of project loads.
const BATCH_MARK: &str = "Completed (re)load of ";

/// Drain incoming messages until the solution is loaded: the server has sent
/// `projectInitializationComplete` and its batch lines add up to `expected`
/// projects. Returns how long it took.
///
/// The signal alone is not enough: Roslyn sends it after the first batch
/// (observed: 1 of 27 projects on `Apex`, 1 of 17 on `MbTx`) while the rest are
/// still loading, so a request right after it hits an unloaded project.
///
/// Every `window/logMessage` and `window/showMessage` on the way is written to
/// `log` as one line, so the run leaves the server's own account behind.
///
/// # Errors
/// [`RoslynError::LoadTimeout`] past `deadline` (with the counts), the server
/// closes, or `log` cannot be written.
pub async fn wait_for_load<W: std::io::Write>(
    incoming: &mut tokio::sync::mpsc::Receiver<Incoming>,
    deadline: Duration,
    expected: usize,
    log: &mut W,
) -> Result<Duration, RoslynError> {
    let started: Instant = Instant::now();
    let mut signalled: bool = false;
    let mut loaded: usize = 0;
    while !(signalled && loaded >= expected) {
        let timeout: RoslynError = RoslynError::LoadTimeout {
            after: deadline,
            loaded,
            expected,
        };
        let Some(remaining): Option<Duration> = deadline.checked_sub(started.elapsed()) else {
            return Err(timeout);
        };
        let next: Option<Incoming> = tokio::time::timeout(remaining, incoming.recv())
            .await
            .map_err(|_: tokio::time::error::Elapsed| timeout)?;
        match next {
            Some(Incoming::Notification { method, .. }) if method == PROJECT_INIT_COMPLETE => {
                signalled = true;
            }
            Some(message) => {
                let text: &str = log_incoming(&message, log)?;
                loaded = loaded.saturating_add(batch_projects(text));
            }
            None => return Err(RoslynError::Client(LspError::Closed)),
        }
    }
    Ok(started.elapsed())
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

/// The project count of a `Completed (re)load of N project(s)` log line,
/// else 0. A progress view sums it the way [`wait_for_load`] does.
#[must_use]
pub fn batch_projects(text: &str) -> usize {
    text.split_once(BATCH_MARK)
        .and_then(|(_, rest): (&str, &str)| rest.split(' ').next())
        .and_then(|n: &str| n.parse::<usize>().ok())
        .unwrap_or(0)
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
