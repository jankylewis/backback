//! Typed MCP results, pages, and errors.

use serde::Deserialize;
use serde_json::json;
use serde_json::Value;

/// An MCP tool advertised by a server.
#[derive(Debug, Deserialize)]
pub struct Tool {
    /// Wire name.
    pub name: String,
    /// Human description.
    #[serde(default)]
    pub description: Option<String>,
    /// JSON schema for arguments.
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// An MCP resource advertised by a server.
#[derive(Debug, Deserialize)]
pub struct Resource {
    /// Resource URI.
    pub uri: String,
    /// Human name.
    pub name: String,
}

/// A tool response. A tool failure is `is_error`, separate from a protocol error.
#[derive(Debug, Deserialize)]
pub struct CallResult {
    /// Content blocks.
    pub content: Vec<Value>,
    /// Whether the tool reported an error.
    #[serde(default, rename = "isError")]
    pub is_error: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ToolPage {
    pub(crate) tools: Vec<Tool>,
    #[serde(default, rename = "nextCursor")]
    pub(crate) next_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ResourcePage {
    pub(crate) resources: Vec<Resource>,
    #[serde(default, rename = "nextCursor")]
    pub(crate) next_cursor: Option<String>,
}

/// Process, transport, and protocol failures.
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    /// OS I/O failure.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// JSON parsing failure.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Request or shutdown exceeded its deadline.
    #[error("MCP {0} timed out")]
    Timeout(&'static str),
    /// Server closed the protocol channel.
    #[error("MCP server exited while waiting for {operation} (status: {status:?})")]
    Exited {
        /// Pending request.
        operation: &'static str,
        /// Observed process status, when available.
        status: Option<String>,
    },
    /// Malformed or mismatched JSON-RPC response.
    #[error("invalid MCP response: {0}")]
    Protocol(String),
    /// Server returned a JSON-RPC error.
    #[error("MCP error {code}: {message}")]
    Remote {
        /// JSON-RPC error code.
        code: i64,
        /// Server message.
        message: String,
    },
}

pub(crate) fn page_params(cursor: Option<&str>) -> Value {
    cursor.map_or_else(|| json!({}), |value: &str| json!({"cursor":value}))
}

pub(crate) fn response(line: &str, id: u64, method: &str) -> Result<Option<Value>, McpError> {
    let envelope: Value = serde_json::from_str(line)?;
    if envelope.get("id").is_none() {
        return Ok(None);
    }
    if envelope.get("jsonrpc") != Some(&json!("2.0")) || envelope.get("id") != Some(&json!(id)) {
        return Err(McpError::Protocol(format!(
            "mismatched envelope for {method}"
        )));
    }
    if let Some(error) = envelope.get("error") {
        return Err(McpError::Remote {
            code: error.get("code").and_then(Value::as_i64).unwrap_or(-1),
            message: error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("missing message")
                .to_owned(),
        });
    }
    envelope
        .get("result")
        .cloned()
        .map(Some)
        .ok_or_else(|| McpError::Protocol("missing result".into()))
}

#[cfg(test)]
mod tests {
    use super::{response, McpError};
    use proptest::prelude::*;
    use serde_json::json;

    #[test]
    fn checks_response_identity_and_error_shape() {
        assert_eq!(
            response(r#"{"jsonrpc":"2.0","method":"notice"}"#, 4, "ping").unwrap(),
            None
        );
        assert_eq!(
            response(r#"{"jsonrpc":"2.0","id":4,"result":{}}"#, 4, "ping").unwrap(),
            Some(json!({}))
        );
        assert!(matches!(
            response(r#"{"jsonrpc":"2.0","id":5,"result":{}}"#, 4, "ping"),
            Err(McpError::Protocol(_))
        ));
        assert!(matches!(
            response(
                r#"{"jsonrpc":"2.0","id":4,"error":{"code":-1,"message":"bad"}}"#,
                4,
                "ping"
            ),
            Err(McpError::Remote { code: -1, .. })
        ));
        assert!(matches!(
            response("garbage", 4, "ping"),
            Err(McpError::Json(_))
        ));
    }

    proptest! {
        #[test]
        fn valid_response_roundtrips_any_id_and_text(id in any::<u64>(), text in ".*") {
            let wire: String = json!({"jsonrpc":"2.0","id":id,"result":{"text":text}}).to_string();
            let result: Option<serde_json::Value> = response(&wire, id, "ping").unwrap();
            prop_assert_eq!(result, Some(json!({"text":text})));
        }
    }
}
