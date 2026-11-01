//! One child process and its newline-delimited JSON-RPC channel.

use std::time::Duration;

use serde_json::{json, Value};
use tokio::process::Child;
use tokio::sync::{broadcast, mpsc};
use tokio::time::timeout;

const DEADLINE: Duration = Duration::from_secs(30);

use crate::types::{CallResult, McpError, Resource, ResourcePage, Tool, ToolPage};
use crate::wire::{self, Inbound, Input};

/// Owns exactly one MCP child and its stdio channel.
pub struct Session {
    pub(crate) child: Child,
    pub(crate) input: Input,
    pub(crate) inbound: mpsc::UnboundedReceiver<Inbound>,
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
                .request("tools/list", wire::page_params(cursor.as_deref()))
                .await?;
            let page: ToolPage = decode(value, "tools/list")?;
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
                .request("resources/list", wire::page_params(cursor.as_deref()))
                .await?;
            let page: ResourcePage = decode(value, "resources/list")?;
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
        decode(value, "tools/call")
    }

    pub(crate) async fn notify(&mut self, method: &'static str) -> Result<(), McpError> {
        let body: Value = json!({"jsonrpc":"2.0","method":method});
        timeout(DEADLINE, wire::write(&self.input, &body))
            .await
            .map_err(|_e: tokio::time::error::Elapsed| timed_out(method))?
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
            wire::write(&self.input, &body).await?;
            loop {
                match self.inbound.recv().await {
                    Some(Inbound::Response(envelope)) => {
                        if let Some(value) = wire::result(&envelope, id, method)? {
                            return Ok(value);
                        }
                    }
                    Some(Inbound::NotJson(first_bytes)) => {
                        return Err(McpError::NotJson { first_bytes });
                    }
                    Some(Inbound::Failed(source)) => {
                        return Err(McpError::Io {
                            operation: "read server stdout",
                            source,
                        });
                    }
                    None => return Err(self.exited(method).await),
                }
            }
        })
        .await
        .map_err(|_e: tokio::time::error::Elapsed| timed_out(method))?
    }

    async fn exited(&mut self, operation: &'static str) -> McpError {
        let status: Option<String> = timeout(Duration::from_secs(2), self.child.wait())
            .await
            .ok()
            .and_then(Result::ok)
            .map(|value: std::process::ExitStatus| value.to_string());
        McpError::Exited { operation, status }
    }
}

fn decode<T: serde::de::DeserializeOwned>(
    value: Value,
    operation: &'static str,
) -> Result<T, McpError> {
    serde_json::from_value(value)
        .map_err(|source: serde_json::Error| McpError::Shape { operation, source })
}

fn timed_out(operation: &'static str) -> McpError {
    McpError::Timeout {
        operation,
        seconds: DEADLINE.as_secs(),
    }
}
