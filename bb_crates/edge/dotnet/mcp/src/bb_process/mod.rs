//! Child process lifecycle and timestamped stderr logging.

mod bb_group;
mod build;

pub(crate) use bb_group::GroupId;
pub use bb_group::{end_all_servers, exit_on_signal};

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{broadcast, Mutex};
use tokio::time::timeout;

use crate::session::Session;
use crate::types::McpError;
use crate::wire::{self, Input};

const STOP_DEADLINE: Duration = Duration::from_secs(2);

impl Session {
    /// Build the project, then run its output with `dotnet` and perform the
    /// MCP handshake. Build output never reaches the protocol channel.
    ///
    /// # Errors
    /// Build, process, or protocol failures.
    pub async fn start(project: &Path) -> Result<Self, McpError> {
        let target: PathBuf = build::build(project).await?;
        Self::launch("dotnet", &[target.into_os_string()]).await
    }

    /// Start any stdio MCP server program, then perform the MCP handshake.
    ///
    /// # Errors
    /// Process or protocol failures.
    pub async fn launch(program: &str, args: &[OsString]) -> Result<Self, McpError> {
        let mut command: Command = Command::new(program);
        command.args(args);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command.kill_on_drop(true);
        #[cfg(unix)]
        command.process_group(0);
        let mut child: Child =
            command
                .spawn()
                .map_err(|source: std::io::Error| McpError::Spawn {
                    program: program.to_owned(),
                    source,
                })?;
        let input: ChildStdin = child
            .stdin
            .take()
            .ok_or_else(|| McpError::Protocol("missing stdin".into()))?;
        let output: ChildStdout = child
            .stdout
            .take()
            .ok_or_else(|| McpError::Protocol("missing stdout".into()))?;
        let stderr: tokio::process::ChildStderr = child
            .stderr
            .take()
            .ok_or_else(|| McpError::Protocol("missing stderr".into()))?;
        let leader: u32 = child
            .id()
            .ok_or_else(|| McpError::Protocol("server exited at spawn".into()))?;
        let group: GroupId = GroupId::register(leader);
        let logs: broadcast::Receiver<String> = capture_logs(stderr);
        let input: Input = Arc::new(Mutex::new(Some(input)));
        let session: Self = Self {
            child,
            inbound: wire::spawn_reader(output, input.clone()),
            input,
            logs,
            next_id: 1,
            group,
        };
        session.handshake().await
    }

    /// `initialize`, then `notifications/initialized`; stop on any failure.
    async fn handshake(mut self) -> Result<Self, McpError> {
        let init: Value = json!({"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"backback","version":"0.0.1"}});
        if let Err(error) = self.request("initialize", init).await {
            let _stopped: Result<std::process::ExitStatus, McpError> = self.stop().await;
            return Err(error);
        }
        if let Err(error) = self.notify("notifications/initialized").await {
            let _stopped: Result<std::process::ExitStatus, McpError> = self.stop().await;
            return Err(error);
        }
        Ok(self)
    }

    /// Subscribe to timestamped stderr lines.
    #[must_use]
    pub fn logs(&mut self) -> &mut broadcast::Receiver<String> {
        &mut self.logs
    }

    /// The operating-system process ID, if the child still exists.
    #[must_use]
    pub fn pid(&self) -> Option<u32> {
        self.child.id()
    }

    /// Stop the server, then kill its process group if it ignores EOF.
    ///
    /// # Errors
    /// Process shutdown failures.
    pub async fn stop(mut self) -> Result<std::process::ExitStatus, McpError> {
        let closed: Option<ChildStdin> = timeout(STOP_DEADLINE, self.input.lock())
            .await
            .map_err(|_e: tokio::time::error::Elapsed| stop_timeout("stdin close"))?
            .take();
        drop(closed);
        if let Ok(result) = timeout(STOP_DEADLINE, self.child.wait()).await {
            return result.map_err(wait_failed);
        }
        self.kill_tree("-TERM").await?;
        if let Ok(result) = timeout(STOP_DEADLINE, self.child.wait()).await {
            return result.map_err(wait_failed);
        }
        self.kill_tree("-KILL").await?;
        timeout(STOP_DEADLINE, self.child.wait())
            .await
            .map_err(|_e: tokio::time::error::Elapsed| stop_timeout("stop"))?
            .map_err(wait_failed)
    }

    /// Stop and start the same project in one explicit operation.
    ///
    /// # Errors
    /// Process or protocol failures.
    pub async fn restart(self, project: &Path) -> Result<Self, McpError> {
        let _status: std::process::ExitStatus = self.stop().await?;
        Self::start(project).await
    }

    async fn kill_tree(&mut self, signal: &str) -> Result<(), McpError> {
        if let Some(pid) = self.child.id() {
            let group: String = format!("-{pid}");
            let mut command: Command = Command::new("/bin/kill");
            command.args([signal, &group]);
            let status: std::process::ExitStatus = timeout(STOP_DEADLINE, command.status())
                .await
                .map_err(|_e: tokio::time::error::Elapsed| stop_timeout("kill tree"))?
                .map_err(|source: std::io::Error| McpError::Io {
                    operation: "kill server process group",
                    source,
                })?;
            let exited: bool = self.child.try_wait().ok().flatten().is_some();
            if !status.success() && !exited {
                return Err(McpError::Protocol(format!(
                    "kill {signal} of process group {pid} failed: {status}"
                )));
            }
        }
        Ok(())
    }
}

fn wait_failed(source: std::io::Error) -> McpError {
    McpError::Io {
        operation: "wait for server exit",
        source,
    }
}

fn stop_timeout(operation: &'static str) -> McpError {
    McpError::Timeout {
        operation,
        seconds: STOP_DEADLINE.as_secs(),
    }
}

fn capture_logs(stderr: tokio::process::ChildStderr) -> broadcast::Receiver<String> {
    let (logs, receiver): (broadcast::Sender<String>, broadcast::Receiver<String>) =
        broadcast::channel(256);
    tokio::spawn(async move {
        let mut lines: tokio::io::Lines<BufReader<tokio::process::ChildStderr>> =
            BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let time: u64 = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d: Duration| d.as_secs());
            let _sent: Result<usize, broadcast::error::SendError<String>> =
                logs.send(format!("{time} {line}"));
        }
    });
    receiver
}
