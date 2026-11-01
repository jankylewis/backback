//! Background stdout reader: answers server requests, forwards responses.

use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout};
use tokio::sync::{mpsc, Mutex};
use tokio::time::timeout;

use crate::types::McpError;

/// The server's stdin; `None` once the client closed it.
pub(crate) type Input = Arc<Mutex<Option<ChildStdin>>>;

const ANSWER_DEADLINE: Duration = Duration::from_secs(5);
const FIRST_BYTES: usize = 80;

/// One message from the server that a pending request must see.
pub(crate) enum Inbound {
    Response(Value),
    NotJson(String),
    Failed(std::io::Error),
}

/// Read stdout until EOF. Server requests are answered at once, at any time.
pub(crate) fn spawn_reader(output: ChildStdout, input: Input) -> mpsc::UnboundedReceiver<Inbound> {
    let (send, receive): (
        mpsc::UnboundedSender<Inbound>,
        mpsc::UnboundedReceiver<Inbound>,
    ) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let mut reader: BufReader<ChildStdout> = BufReader::new(output);
        loop {
            let mut line: String = String::new();
            match reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {}
                Err(error) => {
                    let _closed: bool = send.send(Inbound::Failed(error)).is_err();
                    break;
                }
            }
            if line.trim().is_empty() {
                continue;
            }
            let Ok(message): Result<Value, serde_json::Error> = serde_json::from_str(&line) else {
                let _closed: bool = send.send(Inbound::NotJson(first_bytes(&line))).is_err();
                break;
            };
            if let Some(method) = message.get("method").and_then(Value::as_str) {
                if let Some(id) = message.get("id") {
                    tokio::spawn(answer(input.clone(), id.clone(), method.to_owned()));
                }
                continue;
            }
            if send.send(Inbound::Response(message)).is_err() {
                break;
            }
        }
    });
    receive
}

/// Write one JSON-RPC message as a single line.
pub(crate) async fn write(input: &Input, body: &Value) -> Result<(), McpError> {
    let mut bytes: Vec<u8> = serde_json::to_vec(body).map_err(|source: serde_json::Error| {
        McpError::Protocol(format!("cannot encode request: {source}"))
    })?;
    bytes.push(b'\n');
    let mut guard: tokio::sync::MutexGuard<'_, Option<ChildStdin>> = input.lock().await;
    let pipe: &mut ChildStdin = guard
        .as_mut()
        .ok_or_else(|| McpError::Protocol("server stdin is already closed".into()))?;
    pipe.write_all(&bytes).await.map_err(write_failed)?;
    pipe.flush().await.map_err(write_failed)
}

fn write_failed(source: std::io::Error) -> McpError {
    McpError::Io {
        operation: "write to server stdin",
        source,
    }
}

async fn answer(input: Input, id: Value, method: String) {
    let body: Value = if method == "ping" {
        json!({"jsonrpc":"2.0","id":id,"result":{}})
    } else {
        json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":format!("method not found: {method}")}})
    };
    let _unanswered_server_request_surfaces_as_its_own_timeout: bool =
        timeout(ANSWER_DEADLINE, write(&input, &body))
            .await
            .is_ok_and(|result: Result<(), McpError>| result.is_ok());
}

fn first_bytes(line: &str) -> String {
    line.trim_end().chars().take(FIRST_BYTES).collect()
}

pub(crate) fn page_params(cursor: Option<&str>) -> Value {
    cursor.map_or_else(|| json!({}), |value: &str| json!({"cursor":value}))
}

/// The result for request `id`; `None` for a late reply to an older request.
pub(crate) fn result(envelope: &Value, id: u64, method: &str) -> Result<Option<Value>, McpError> {
    let got: Option<&Value> = envelope.get("id");
    if got
        .and_then(Value::as_u64)
        .is_some_and(|older: u64| older < id)
    {
        return Ok(None);
    }
    if envelope.get("jsonrpc") != Some(&json!("2.0")) || got != Some(&json!(id)) {
        return Err(McpError::Protocol(format!(
            "mismatched envelope for {method}: expected id {id}, got {}",
            got.map_or_else(|| "none".to_owned(), Value::to_string)
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
        .ok_or_else(|| McpError::Protocol(format!("missing result for {method}")))
}

#[cfg(test)]
mod tests {
    use super::{result, McpError};
    use proptest::prelude::*;
    use serde_json::json;

    #[test]
    fn checks_response_identity_staleness_and_error_shape() {
        assert_eq!(
            result(&json!({"jsonrpc":"2.0","id":4,"result":{}}), 4, "ping").unwrap(),
            Some(json!({}))
        );
        assert_eq!(
            result(&json!({"jsonrpc":"2.0","id":3,"result":{}}), 4, "ping").unwrap(),
            None
        );
        assert!(matches!(
            result(&json!({"jsonrpc":"2.0","id":5,"result":{}}), 4, "ping"),
            Err(McpError::Protocol(text)) if text.contains("expected id 4, got 5")
        ));
        assert!(matches!(
            result(
                &json!({"jsonrpc":"2.0","id":4,"error":{"code":-1,"message":"bad"}}),
                4,
                "ping"
            ),
            Err(McpError::Remote { code: -1, .. })
        ));
        assert!(matches!(
            result(&json!({"jsonrpc":"2.0","id":4}), 4, "ping"),
            Err(McpError::Protocol(text)) if text.contains("missing result for ping")
        ));
    }

    proptest! {
        #[test]
        fn valid_response_roundtrips_any_id_and_text(id in any::<u64>(), text in ".*") {
            let wire: serde_json::Value = json!({"jsonrpc":"2.0","id":id,"result":{"text":text}});
            let value: Option<serde_json::Value> = result(&wire, id, "ping").unwrap();
            prop_assert_eq!(value, Some(json!({"text":text})));
        }
    }
}
