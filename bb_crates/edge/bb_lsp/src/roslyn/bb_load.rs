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
            Some(Incoming::Notification { method, params }) => {
                let text: &str = log_message(&method, params.as_ref(), log)?;
                loaded = loaded.saturating_add(batch_size(text));
            }
            Some(Incoming::ServerRequest { method, params }) => {
                log_configuration(&method, params.as_ref(), log)?;
            }
            None => return Err(RoslynError::Client(LspError::Closed)),
        }
    }
    Ok(started.elapsed())
}

/// The project count of a batch line, else 0.
fn batch_size(text: &str) -> usize {
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
