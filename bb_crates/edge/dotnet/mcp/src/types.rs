//! Typed MCP results, pages, and errors.

use serde::Deserialize;
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

/// Process, transport, and protocol failures. Every message names its cause.
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    /// The server program could not start.
    #[error("cannot start `{program}`: {}", spawn_cause(.source))]
    Spawn {
        /// Program name.
        program: String,
        /// OS failure.
        source: std::io::Error,
    },
    /// `dotnet build` of the server project failed.
    #[error("`dotnet build` failed for {project} ({status}):\n{output}")]
    Build {
        /// Project path.
        project: String,
        /// Exit status.
        status: String,
        /// Last lines of build output.
        output: String,
    },
    /// OS I/O failure during one named operation.
    #[error("MCP {operation} failed: {source}")]
    Io {
        /// What was running.
        operation: &'static str,
        /// OS failure.
        source: std::io::Error,
    },
    /// The server wrote a line that is not JSON to the protocol channel.
    #[error("MCP server wrote non-JSON to stdout, which is the protocol channel (log to stderr); first bytes: {first_bytes:?}")]
    NotJson {
        /// Start of the bad line.
        first_bytes: String,
    },
    /// A result did not have the shape MCP defines.
    #[error("invalid MCP {operation} result: {source}")]
    Shape {
        /// Request method.
        operation: &'static str,
        /// Decode failure.
        source: serde_json::Error,
    },
    /// Request or shutdown exceeded its deadline.
    #[error("MCP {operation} timed out after {seconds} s")]
    Timeout {
        /// What was running.
        operation: &'static str,
        /// Deadline.
        seconds: u64,
    },
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

fn spawn_cause(source: &std::io::Error) -> String {
    if source.kind() == std::io::ErrorKind::NotFound {
        "not found on PATH (install it or fix PATH)".to_owned()
    } else {
        source.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::McpError;

    fn process_error_cases() -> Vec<(McpError, &'static str)> {
        let bad_json: serde_json::Error = serde_json::from_str::<u8>("x").unwrap_err();
        vec![
            (
                McpError::Spawn {
                    program: "dotnet".into(),
                    source: std::io::ErrorKind::NotFound.into(),
                },
                "cannot start `dotnet`: not found on PATH",
            ),
            (
                McpError::Build {
                    project: "a.csproj".into(),
                    status: "exit status: 1".into(),
                    output: "CS0029".into(),
                },
                "`dotnet build` failed for a.csproj",
            ),
            (
                McpError::Io {
                    operation: "write to server stdin",
                    source: std::io::ErrorKind::BrokenPipe.into(),
                },
                "MCP write to server stdin failed",
            ),
            (
                McpError::NotJson {
                    first_bytes: "Hello".into(),
                },
                "non-JSON to stdout",
            ),
            (
                McpError::Shape {
                    operation: "tools/list",
                    source: bad_json,
                },
                "invalid MCP tools/list result",
            ),
        ]
    }

    fn protocol_error_cases() -> Vec<(McpError, &'static str)> {
        vec![
            (
                McpError::Timeout {
                    operation: "initialize",
                    seconds: 30,
                },
                "MCP initialize timed out after 30 s",
            ),
            (
                McpError::Exited {
                    operation: "tools/call",
                    status: None,
                },
                "exited while waiting for tools/call",
            ),
            (McpError::Protocol("x".into()), "invalid MCP response: x"),
            (
                McpError::Remote {
                    code: -32601,
                    message: "nope".into(),
                },
                "MCP error -32601: nope",
            ),
        ]
    }

    #[test]
    fn every_error_message_names_its_cause() {
        for (error, wanted) in process_error_cases()
            .into_iter()
            .chain(protocol_error_cases())
        {
            let text: String = error.to_string();
            assert!(text.contains(wanted), "{text:?} lacks {wanted:?}");
        }
    }
}
