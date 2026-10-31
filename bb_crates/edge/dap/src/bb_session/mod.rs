//! A bounded DAP connection to one netcoredbg process.

mod bb_reader;
mod commands;
mod inspect;
mod state;
pub use commands::{GotoTarget, Step};

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{mpsc, oneshot};

use crate::{DapError, Event, Message, DAP_ARGS};
use state::{Action, SessionState};

type Reply = Result<Value, DapError>;
type Pending = Arc<Mutex<HashMap<u64, oneshot::Sender<Reply>>>>;

/// One running debug adapter. Dropping it kills the child.
pub struct DapSession {
    child: Child,
    outgoing: mpsc::Sender<Vec<u8>>,
    pending: Pending,
    closed: Arc<AtomicBool>,
    next_seq: AtomicU64,
    events: Option<mpsc::Receiver<Event>>,
    timeout: Duration,
    capabilities: Mutex<Option<crate::Capabilities>>,
    state: Mutex<SessionState>,
}

impl DapSession {
    /// Spawn the installed adapter in VS Code DAP mode.
    ///
    /// # Errors
    /// Missing adapter or process spawn failure.
    pub fn spawn(timeout: Duration) -> Result<Self, DapError> {
        let path: std::path::PathBuf = crate::locate()?;
        let mut command: Command = Command::new(path);
        command
            .args(DAP_ARGS)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let mut child: Child = command.spawn().map_err(DapError::Spawn)?;
        let stdin: ChildStdin = child.stdin.take().ok_or(DapError::NoPipes)?;
        let stdout: tokio::process::ChildStdout = child.stdout.take().ok_or(DapError::NoPipes)?;
        let (outgoing, rx): (mpsc::Sender<Vec<u8>>, mpsc::Receiver<Vec<u8>>) = mpsc::channel(64);
        tokio::spawn(write_loop(stdin, rx));
        let pending: Pending = Arc::default();
        let closed: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let (event_tx, event_rx): (mpsc::Sender<Event>, mpsc::Receiver<Event>) = mpsc::channel(256);
        tokio::spawn(bb_reader::run(
            stdout,
            Arc::clone(&pending),
            Arc::clone(&closed),
            event_tx,
        ));
        Ok(Self {
            child,
            outgoing,
            pending,
            closed,
            next_seq: AtomicU64::new(1),
            events: Some(event_rx),
            timeout,
            capabilities: Mutex::new(None),
            state: Mutex::new(SessionState::Created),
        })
    }

    /// Take the event stream once.
    pub fn take_events(&mut self) -> Option<mpsc::Receiver<Event>> {
        self.events.take()
    }

    /// Whether Set Next Statement can appear in the user interface.
    pub fn can_set_next_statement(&self) -> bool {
        self.capabilities
            .lock()
            .ok()
            .and_then(
                |guard: std::sync::MutexGuard<'_, Option<crate::Capabilities>>| {
                    guard
                        .as_ref()
                        .map(|caps: &crate::Capabilities| caps.supports_goto_targets_request)
                },
            )
            .unwrap_or(false)
    }

    fn require_action(&self, action: Action) -> Result<SessionState, DapError> {
        self.state
            .lock()
            .map_err(|_: _| DapError::Closed)?
            .next(action)
    }

    fn commit_state(&self, next: SessionState) -> Result<(), DapError> {
        *self.state.lock().map_err(|_: _| DapError::Closed)? = next;
        Ok(())
    }

    /// Send a DAP command with a deadline and decode its result body.
    ///
    /// # Errors
    /// Timeout, adapter exit, rejected command, or invalid result.
    pub async fn request<R: DeserializeOwned>(
        &self,
        command: &str,
        arguments: Value,
    ) -> Result<R, DapError> {
        let seq: u64 = self.next_seq.fetch_add(1, Ordering::Relaxed);
        if self.closed.load(Ordering::Acquire) {
            return Err(DapError::Closed);
        }
        let (tx, rx): (oneshot::Sender<Reply>, oneshot::Receiver<Reply>) = oneshot::channel();
        self.pending
            .lock()
            .map_err(|_: _| DapError::Closed)?
            .insert(seq, tx);
        if self.closed.load(Ordering::Acquire) {
            self.pending
                .lock()
                .map_err(|_: _| DapError::Closed)?
                .remove(&seq);
            return Err(DapError::Closed);
        }
        let frame: Vec<u8> = Message::Request {
            seq,
            command: command.to_owned(),
            arguments: Some(arguments),
        }
        .encode()?;
        let outcome: Result<Result<Reply, DapError>, tokio::time::error::Elapsed> =
            tokio::time::timeout(self.timeout, async {
                self.outgoing
                    .send(frame)
                    .await
                    .map_err(|_: _| DapError::Closed)?;
                rx.await.map_err(|_: _| DapError::Closed)
            })
            .await;
        self.pending
            .lock()
            .map_err(|_: _| DapError::Closed)?
            .remove(&seq);
        let body: Value = outcome.map_err(|_: _| DapError::Timeout {
            command: command.to_owned(),
            after: self.timeout,
        })???;
        serde_json::from_value(body).map_err(DapError::Envelope)
    }

    /// Wait for one event within the session deadline.
    ///
    /// # Errors
    /// The deadline passed or the adapter closed.
    pub async fn next_event(
        rx: &mut mpsc::Receiver<Event>,
        timeout: Duration,
    ) -> Result<Event, DapError> {
        tokio::time::timeout(timeout, rx.recv())
            .await
            .map_err(|_: _| DapError::Timeout {
                command: "event".to_owned(),
                after: timeout,
            })?
            .ok_or(DapError::Closed)
    }

    /// Wait briefly for the adapter to exit after a disconnect.
    ///
    /// # Errors
    /// Adapter exit exceeded the deadline.
    pub async fn wait_exit(&mut self) -> Result<(), DapError> {
        tokio::time::timeout(self.timeout, self.child.wait())
            .await
            .map_err(|_: _| DapError::Timeout {
                command: "exit".to_owned(),
                after: self.timeout,
            })?
            .map_err(DapError::Spawn)?;
        Ok(())
    }
}

async fn write_loop(mut stdin: ChildStdin, mut rx: mpsc::Receiver<Vec<u8>>) {
    while let Some(frame) = rx.recv().await {
        if stdin.write_all(&frame).await.is_err() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn real_adapter_hides_unsupported_goto_and_closes_waiters() {
        if crate::locate().is_err() {
            return;
        }
        let mut session: DapSession = DapSession::spawn(Duration::from_secs(10)).unwrap();
        let caps: crate::Capabilities = session.initialize().await.unwrap();
        assert!(!caps.supports_goto_targets_request);
        assert!(!session.can_set_next_statement());
        assert!(matches!(
            session.goto(1, 1).await,
            Err(DapError::Unsupported(_))
        ));
        session.child.start_kill().unwrap();
        session.wait_exit().await.unwrap();
        assert!(matches!(session.threads().await, Err(DapError::Closed)));
    }
}
