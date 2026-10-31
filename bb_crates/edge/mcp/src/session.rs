//! One child process and its newline-delimited JSON-RPC channel.

use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::broadcast;
use tokio::time::timeout;

const DEADLINE: Duration = Duration::from_secs(30);

use crate::types::{CallResult, McpError, Resource, ResourcePage, Tool, ToolPage};

/// Owns exactly one MCP child and its stdio channel.
pub struct Session {
    pub(crate) child: Child,
    pub(crate) input: ChildStdin,
    pub(crate) output: BufReader<ChildStdout>,
    pub(crate) logs: broadcast::Receiver<String>,
    pub(crate) next_id: u64,
}

impl Session {
    /// Probe server liveness.
    ///
    /// # Errors
    /// Process or protocol failures.
    pub async fn ping(&mut self) -> Result<(), McpError> {
        self.request("ping", json!({}))
            .await
            .map(|_value: Value| ())
    }

    /// List all tool pages.
    ///
    /// # Errors
    /// Process or protocol failures.
    pub async fn tools(&mut self) -> Result<Vec<Tool>, McpError> {
        let mut all: Vec<Tool> = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let value: Value = self
                .request("tools/list", crate::types::page_params(cursor.as_deref()))
                .await?;
            let page: ToolPage = serde_json::from_value(value)?;
            all.extend(page.tools);
            cursor = page.next_cursor;
            if cursor.is_none() {
                return Ok(all);
            }
        }
    }

    /// List all resource pages.
    ///
    /// # Errors
    /// Process or protocol failures.
    pub async fn resources(&mut self) -> Result<Vec<Resource>, McpError> {
        let mut all: Vec<Resource> = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let value: Value = self
                .request(
                    "resources/list",
                    crate::types::page_params(cursor.as_deref()),
                )
                .await?;
            let page: ResourcePage = serde_json::from_value(value)?;
            all.extend(page.resources);
            cursor = page.next_cursor;
            if cursor.is_none() {
                return Ok(all);
            }
        }
    }

    /// Call one tool with JSON arguments.
    ///
    /// # Errors
    /// Process or protocol failures.
    pub async fn call(&mut self, name: &str, arguments: Value) -> Result<CallResult, McpError> {
        let value: Value = self
            .request("tools/call", json!({"name":name,"arguments":arguments}))
            .await?;
        Ok(serde_json::from_value(value)?)
    }

    pub(crate) async fn notify(&mut self, method: &str) -> Result<(), McpError> {
        let body: Value = json!({"jsonrpc":"2.0","method":method});
        timeout(DEADLINE, self.write(&body))
            .await
            .map_err(|_e: tokio::time::error::Elapsed| McpError::Timeout("notification"))?
    }

    pub(crate) async fn request(
        &mut self,
        method: &'static str,
        params: Value,
    ) -> Result<Value, McpError> {
        let id: u64 = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| McpError::Protocol("request id overflow".into()))?;
        let body: Value = json!({"jsonrpc":"2.0","id":id,"method":method,"params":params});
        timeout(DEADLINE, async {
            self.write(&body).await?;
            loop {
                let mut line: String = String::new();
                if self.output.read_line(&mut line).await? == 0 {
                    let status: Option<String> = timeout(Duration::from_secs(2), self.child.wait())
                        .await
                        .ok()
                        .and_then(Result::ok)
                        .map(|value: std::process::ExitStatus| value.to_string());
                    return Err(McpError::Exited {
                        operation: method,
                        status,
                    });
                }
                if let Some(value) = crate::types::response(&line, id, method)? {
                    return Ok(value);
                }
            }
        })
        .await
        .map_err(|_e: tokio::time::error::Elapsed| McpError::Timeout(method))?
    }

    async fn write(&mut self, body: &Value) -> Result<(), McpError> {
        let mut bytes: Vec<u8> = serde_json::to_vec(body)?;
        bytes.push(b'\n');
        self.input.write_all(&bytes).await?;
        self.input.flush().await?;
        Ok(())
    }
}
