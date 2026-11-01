//! A bounded DAP connection to one netcoredbg process.

mod bb_reader;
mod commands;
mod inspect;
mod lifecycle;
pub use commands::{GotoTarget, Step};

use bb_process::OwnedGroup;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{mpsc, oneshot};

use crate::{AdapterExit, DapError, Event, Message, DAP_ARGS};
use lifecycle::state::{Action, SessionState};
use lifecycle::{Liveness, Target, Watcher};

type Reply = Result<Value, DapError>;
type Pending = Arc<Mutex<HashMap<u64, oneshot::Sender<Reply>>>>;

/// One running debug adapter.
///
/// Liveness comes from the adapter process exit: when it exits, every waiter
/// fails with [`DapError::Closed`] at once and the event stream closes. A
/// launched debuggee is ended with the adapter; an attached one never is.
/// Dropping the session stops the adapter and ends a launched debuggee.
pub struct DapSession {
    watcher: Watcher,
    live: Liveness,
    outgoing: mpsc::Sender<Vec<u8>>,
    next_seq: AtomicU64,
    events: Option<mpsc::Receiver<Event>>,
    events_weak: mpsc::WeakSender<Event>,
    timeout: Duration,
    capabilities: Mutex<Option<crate::Capabilities>>,
    state: Mutex<SessionState>,
}

impl DapSession {
    /// Spawn the installed adapter in VS Code DAP mode, in its own process group.
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
            .process_group(0)
            .kill_on_drop(true);
        let mut child: Child = command.spawn().map_err(DapError::Spawn)?;
        let group: Option<OwnedGroup> = child
            .id()
            .and_then(NonZeroU32::new)
            .map(OwnedGroup::register);
        let stdin: ChildStdin = child.stdin.take().ok_or(DapError::NoPipes)?;
        let stdout: tokio::process::ChildStdout = child.stdout.take().ok_or(DapError::NoPipes)?;
        let (outgoing, rx): (mpsc::Sender<Vec<u8>>, mpsc::Receiver<Vec<u8>>) = mpsc::channel(64);
        tokio::spawn(write_loop(stdin, rx));
        let live: Liveness = Liveness::new();
        let (event_tx, event_rx): (mpsc::Sender<Event>, mpsc::Receiver<Event>) = mpsc::channel(256);
        let events_weak: mpsc::WeakSender<Event> = event_tx.downgrade();
        let reader: tokio::task::JoinHandle<()> =
            tokio::spawn(bb_reader::run(stdout, live.clone(), event_tx));
        let watcher: Watcher = Watcher::spawn(child, group, reader.abort_handle(), live.clone());
        Ok(Self {
            watcher,
            live,
            outgoing,
            next_seq: AtomicU64::new(1),
            events: Some(event_rx),
            events_weak,
            timeout,
            capabilities: Mutex::new(None),
            state: Mutex::new(SessionState::Created),
        })
    }

    /// Operating system process ID of the adapter.
    pub fn adapter_pid(&self) -> Option<u32> {
        self.watcher.pid()
    }

    /// How the adapter ended, once it has exited and the cleanup is done.
    pub fn adapter_exit(&self) -> Option<AdapterExit> {
        self.watcher.exit()
    }

    /// Kill the adapter now; waiters then fail with [`DapError::Closed`].
    pub fn stop_adapter(&mut self) {
        self.watcher.stop();
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
        if self.live.is_closed() {
            return Err(DapError::Closed);
        }
        let (tx, rx): (oneshot::Sender<Reply>, oneshot::Receiver<Reply>) = oneshot::channel();
        self.live
            .pending
            .lock()
            .map_err(|_: _| DapError::Closed)?
            .insert(seq, tx);
        if self.live.is_closed() {
            self.live
                .pending
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
        self.live
            .pending
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

    /// Wait for the adapter to exit after a disconnect, and for the cleanup.
    ///
    /// # Errors
    /// Adapter exit exceeded the deadline.
    pub async fn wait_exit(&mut self) -> Result<AdapterExit, DapError> {
        self.watcher.wait(self.timeout).await
    }

    fn set_target(&self, target: Target) -> Result<(), DapError> {
        self.live.set_target(target)?;
        if let Target::Attached(pid) = target {
            lifecycle::watch_target(pid, self.events_weak.clone(), self.live.clone());
        }
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
