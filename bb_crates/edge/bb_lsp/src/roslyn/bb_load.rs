//! Waiting for the solution load, and the run log it leaves behind.

use std::time::{Duration, Instant};

use super::{RoslynError, PROJECT_INIT_COMPLETE};
use crate::bb_client::{Incoming, LspError};

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
            Some(Incoming::ServerRequest { method, params }) => {
                log_configuration(&method, params.as_ref(), log)?;
            }
            None => return Err(RoslynError::Client(LspError::Closed)),
        }
    }
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
