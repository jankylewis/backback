//! Generic JSON-RPC 2.0 client over a child process's stdio.

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{mpsc, oneshot, watch};

use crate::wire;

#[path = "client_child/bb_child.rs"]
mod bb_child;

/// Client errors. Every request either returns a value or one of these.
#[derive(Debug, thiserror::Error)]
pub enum LspError {
    /// The server process could not be started.
    #[error("cannot spawn language server: {0}")]
    Spawn(#[source] std::io::Error),
    /// The server did not expose stdin/stdout pipes.
    #[error("language server has no stdio pipes")]
    NoPipes,
    /// The deadline for a request passed.
    #[error("request {method} timed out after {after:?}")]
    Timeout {
        /// The request method.
        method: String,
        /// The deadline that passed.
        after: Duration,
    },
    /// The connection closed before a response arrived.
    #[error("language server closed the connection")]
    Closed,
    /// The server answered with a JSON-RPC error.
    #[error("server error {code}: {message}")]
    Server {
        /// JSON-RPC error code.
        code: i64,
        /// Server message.
        message: String,
    },
    /// A result did not have the expected shape.
    #[error("unexpected response shape: {0}")]
    Json(#[from] serde_json::Error),
}

/// A message that arrived from the server and is not a response to us.
#[derive(Debug, Clone)]
pub enum Incoming {
    /// A notification (`method` + `params`).
    Notification {
        /// LSP method name.
        method: String,
        /// Parameters, if any.
        params: Option<Value>,
    },
    /// A server-to-client request. The client already replied `null`; this is
    /// for visibility (for example `window/workDoneProgress/create`).
    ServerRequest {
        /// LSP method name.
        method: String,
        /// Parameters, if any.
        params: Option<Value>,
    },
}

/// Answers for `workspace/configuration`: section name to value.
pub type Settings = serde_json::Map<String, Value>;

/// What a waiter receives: the result value or the error.
pub(crate) type Reply = Result<Value, LspError>;

/// Waiters for responses, keyed by request id.
pub(crate) type Pending = Arc<Mutex<HashMap<i64, oneshot::Sender<Reply>>>>;

/// A running language server connection.
pub struct LspClient {
    child: Child,
    outgoing: mpsc::Sender<Vec<u8>>,
    pending: Pending,
    next_id: AtomicI64,
    incoming: Option<mpsc::Receiver<Incoming>>,
    closed: watch::Receiver<bool>,
    timeout: Duration,
}

impl LspClient {
    /// Spawn `command` and wire its stdio to this client. `settings` answers
    /// the server's `workspace/configuration` requests.
    ///
    /// # Errors
    /// [`LspError::Spawn`] or [`LspError::NoPipes`].
    pub fn spawn(
        mut command: Command,
        timeout: Duration,
        settings: Settings,
    ) -> Result<Self, LspError> {
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true);
        let mut child: Child = command.spawn().map_err(LspError::Spawn)?;
        let stdin: ChildStdin = child.stdin.take().ok_or(LspError::NoPipes)?;
        let stdout: ChildStdout = child.stdout.take().ok_or(LspError::NoPipes)?;
        let (outgoing, outgoing_rx): (mpsc::Sender<Vec<u8>>, mpsc::Receiver<Vec<u8>>) =
            mpsc::channel(64);
        tokio::spawn(wire::write_loop(stdin, outgoing_rx));
        let pending: Pending = Arc::default();
        let (incoming_tx, incoming_rx): (mpsc::Sender<Incoming>, mpsc::Receiver<Incoming>) =
            mpsc::channel(256);
        let reader: wire::Reader = wire::Reader {
            pending: Arc::clone(&pending),
            incoming: incoming_tx,
            outgoing: outgoing.clone(),
            settings: Arc::new(settings),
            closed: watch::channel(false).0,
        };
        let closed: watch::Receiver<bool> = reader.closed.subscribe();
        tokio::spawn(reader.run(stdout));
        Ok(Self {
            child,
            outgoing,
            pending,
            next_id: AtomicI64::new(1),
            incoming: Some(incoming_rx),
            closed,
            timeout,
        })
    }

    /// Take the stream of server notifications and requests. Can be taken once.
    pub fn take_incoming(&mut self) -> Option<mpsc::Receiver<Incoming>> {
        self.incoming.take()
    }

    /// Observe EOF on the server connection without waiting for a request.
    #[must_use]
    pub fn closed_signal(&self) -> watch::Receiver<bool> {
        self.closed.clone()
    }

    /// Send a request and wait (bounded by the client's deadline) for its result.
    ///
    /// # Errors
    /// Timeout, closed connection, server error, or a result of the wrong shape.
    pub async fn request<P: Serialize, R: DeserializeOwned>(
        &self,
        method: &str,
        params: P,
    ) -> Result<R, LspError> {
        self.request_within(method, params, self.timeout).await
    }

    /// Send a request and wait at most `deadline` for its result. A request
    /// that times out is forgotten, so a late answer cannot reach a new waiter.
    ///
    /// # Errors
    /// Timeout, closed connection, server error, or a result of the wrong shape.
    pub async fn request_within<P: Serialize, R: DeserializeOwned>(
        &self,
        method: &str,
        params: P,
        deadline: Duration,
    ) -> Result<R, LspError> {
        let id: i64 = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx): (oneshot::Sender<Reply>, oneshot::Receiver<Reply>) = oneshot::channel();
        // A poisoned map can never deliver the reply: fail now, not at the
        // deadline.
        self.pending
            .lock()
            .map_err(|_: std::sync::PoisonError<_>| LspError::Closed)?
            .insert(id, tx);
        let message: Value =
            json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        self.send(&message).await?;
        let waited: Result<Result<Reply, oneshot::error::RecvError>, tokio::time::error::Elapsed> =
            tokio::time::timeout(deadline, rx).await;
        let Ok(received): Result<
            Result<Reply, oneshot::error::RecvError>,
            tokio::time::error::Elapsed,
        > = waited
        else {
            self.forget(id);
            return Err(LspError::Timeout {
                method: method.to_owned(),
                after: deadline,
            });
        };
        let reply: Value = received.map_err(|_: oneshot::error::RecvError| LspError::Closed)??;
        Ok(serde_json::from_value::<R>(reply)?)
    }

    /// Send a notification (no response expected).
    ///
    /// # Errors
    /// [`LspError::Closed`] when the writer has stopped.
    pub async fn notify<P: Serialize>(&self, method: &str, params: P) -> Result<(), LspError> {
        let message: Value = json!({ "jsonrpc": "2.0", "method": method, "params": params });
        self.send(&message).await
    }

    /// Ask the server to exit: `shutdown` request, `exit` notification, then wait.
    ///
    /// # Errors
    /// [`LspError::Closed`] when `exit` cannot be sent.
    pub async fn shutdown(mut self) -> Result<(), LspError> {
        // A failed `shutdown` must not stop `exit`: the process is leaving either way.
        let _ignored: Result<Value, LspError> = self.request("shutdown", Value::Null).await;
        self.notify("exit", Value::Null).await?;
        let _status: Result<
            std::io::Result<std::process::ExitStatus>,
            tokio::time::error::Elapsed,
        > = tokio::time::timeout(self.timeout, self.child.wait()).await;
        Ok(())
    }

    /// Requests still waiting for an answer.
    #[cfg(test)]
    pub(crate) fn pending_len(&self) -> usize {
        self.pending.lock().map_or(
            0,
            |map: std::sync::MutexGuard<'_, HashMap<i64, oneshot::Sender<Reply>>>| map.len(),
        )
    }

    /// Drop the waiter of a request that will not be awaited any more.
    fn forget(&self, id: i64) {
        if let Ok(mut map) = self.pending.lock() {
            map.remove(&id);
        }
    }

    async fn send(&self, message: &Value) -> Result<(), LspError> {
        self.outgoing
            .send(bb_framing::encode(message))
            .await
            .map_err(|_: mpsc::error::SendError<Vec<u8>>| LspError::Closed)
    }
}
