//! Child process lifecycle and timestamped stderr logging.

use std::path::Path;
use std::process::Stdio;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::broadcast;
use tokio::time::timeout;

use crate::session::Session;
use crate::types::McpError;

const STOP_DEADLINE: Duration = Duration::from_secs(2);

impl Session {
    /// Start `dotnet run --project`, then perform the MCP handshake.
    ///
    /// # Errors
    /// Process or protocol failures.
    pub async fn start(project: &Path) -> Result<Self, McpError> {
        let mut command: Command = Command::new("dotnet");
        command.args(["run", "--no-launch-profile", "--project"]);
        command.arg(project);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command.kill_on_drop(true);
        #[cfg(unix)]
        command.process_group(0);
        let mut child: Child = command.spawn()?;
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
        let logs: broadcast::Receiver<String> = capture_logs(stderr);
        let mut session: Self = Self {
            child,
            input,
            output: BufReader::new(output),
            logs,
            next_id: 1,
        };
        let init: Value = json!({"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"backback","version":"0.0.1"}});
        if let Err(error) = session.request("initialize", init).await {
            let _stopped: Result<std::process::ExitStatus, McpError> = session.stop().await;
            return Err(error);
        }
        if let Err(error) = session.notify("notifications/initialized").await {
            let _stopped: Result<std::process::ExitStatus, McpError> = session.stop().await;
            return Err(error);
        }
        Ok(session)
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
        timeout(STOP_DEADLINE, self.input.shutdown())
            .await
            .map_err(|_e: tokio::time::error::Elapsed| McpError::Timeout("stdin close"))??;
        if let Ok(result) = timeout(STOP_DEADLINE, self.child.wait()).await {
            return Ok(result?);
        }
        self.kill_tree("-TERM").await?;
        if let Ok(result) = timeout(STOP_DEADLINE, self.child.wait()).await {
            return Ok(result?);
        }
        self.kill_tree("-KILL").await?;
        let status: std::process::ExitStatus =
            timeout(STOP_DEADLINE, self.child.wait())
                .await
                .map_err(|_e: tokio::time::error::Elapsed| McpError::Timeout("stop"))??;
        Ok(status)
    }

    /// Stop and start the same project in one explicit operation.
    ///
    /// # Errors
    /// Process or protocol failures.
    pub async fn restart(self, project: &Path) -> Result<Self, McpError> {
        let _status: std::process::ExitStatus = self.stop().await?;
        Self::start(project).await
    }

    async fn kill_tree(&self, signal: &str) -> Result<(), McpError> {
        if let Some(pid) = self.child.id() {
            let group: String = format!("-{pid}");
            let mut command: Command = Command::new("/bin/kill");
            command.args([signal, &group]);
            let status: std::process::ExitStatus = timeout(STOP_DEADLINE, command.status())
                .await
                .map_err(|_e: tokio::time::error::Elapsed| McpError::Timeout("kill tree"))??;
            if !status.success() {
                return Err(McpError::Protocol(format!("kill tree failed: {status}")));
            }
        }
        Ok(())
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
