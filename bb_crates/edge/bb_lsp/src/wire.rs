//! Reader side of the connection: decode frames, route responses to waiters,
//! answer `workspace/configuration` from the client's settings and every
//! other server request with `null`, forward notifications.

use serde_json::{json, Value};
use tokio::io::AsyncReadExt;
use tokio::process::ChildStdout;
use tokio::sync::{mpsc, oneshot};

use bb_framing::{Decoder, FrameError};

use std::sync::Arc;

use crate::bb_client::{Incoming, LspError, Pending, Settings};

/// Everything the reader task needs.
pub(crate) struct Reader {
    pub(crate) pending: Pending,
    pub(crate) incoming: mpsc::Sender<Incoming>,
    pub(crate) outgoing: mpsc::Sender<Vec<u8>>,
    pub(crate) settings: Arc<Settings>,
}

impl Reader {
    /// Read until EOF, then fail every waiter so nobody hangs on a dead server.
    pub(crate) async fn run(self, mut stdout: ChildStdout) {
        let mut decoder: Decoder = Decoder::default();
        let mut chunk: Vec<u8> = vec![0u8; 16 * 1024];
        while let Ok(n) = stdout.read(&mut chunk).await {
            if n == 0 {
                break;
            }
            if let Some(bytes) = chunk.get(..n) {
                decoder.push(bytes);
            }
            self.drain(&mut decoder).await;
        }
        if let Ok(mut map) = self.pending.lock() {
            for (_, tx) in map.drain() {
                let _dropped: Result<(), Result<Value, LspError>> = tx.send(Err(LspError::Closed));
            }
        }
    }

    async fn drain(&self, decoder: &mut Decoder) {
        loop {
            let next: Result<Option<Value>, FrameError> = decoder.next_message();
            match next {
                Ok(Some(message)) => self.dispatch(message).await,
                Ok(None) => break,
                Err(_) => {} // the decoder already dropped the bad frame
            }
        }
    }

    async fn dispatch(&self, message: Value) {
        let method: Option<String> = message
            .get("method")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let id: Option<Value> = message.get("id").cloned();
        let params: Option<Value> = message.get("params").cloned();
        match (method, id) {
            (None, Some(id)) => self.resolve(&id, &message),
            (Some(method), Some(id)) => {
                let result: Value = if method == "workspace/configuration" {
                    configuration_reply(params.as_ref(), &self.settings)
                } else {
                    Value::Null
                };
                let reply: Value = json!({ "jsonrpc": "2.0", "id": id, "result": result });
                let _sent: Result<(), mpsc::error::SendError<Vec<u8>>> =
                    self.outgoing.send(bb_framing::encode(&reply)).await;
                let _sent: Result<(), mpsc::error::SendError<Incoming>> = self
                    .incoming
                    .send(Incoming::ServerRequest { method, params })
                    .await;
            }
            (Some(method), None) => {
                let _sent: Result<(), mpsc::error::SendError<Incoming>> = self
                    .incoming
                    .send(Incoming::Notification { method, params })
                    .await;
            }
            (None, None) => {}
        }
    }

    fn resolve(&self, id: &Value, message: &Value) {
        let Some(id): Option<i64> = id.as_i64() else {
            return;
        };
        let outcome: Result<Value, LspError> = response_outcome(message);
        let waiter: Option<oneshot::Sender<Result<Value, LspError>>> = match self.pending.lock() {
            Ok(mut map) => map.remove(&id),
            Err(_) => None,
        };
        if let Some(tx) = waiter {
            let _dropped: Result<(), Result<Value, LspError>> = tx.send(outcome);
        }
    }
}

/// One value per requested item: the setting for its `section`, else `null`.
pub(crate) fn configuration_reply(params: Option<&Value>, settings: &Settings) -> Value {
    let items: &[Value] = params
        .and_then(|p: &Value| p.get("items"))
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice);
    items
        .iter()
        .map(|item: &Value| {
            item.get("section")
                .and_then(Value::as_str)
                .and_then(|section: &str| settings.get(section))
                .cloned()
                .unwrap_or(Value::Null)
        })
        .collect()
}

/// `result` becomes `Ok`, `error` becomes [`LspError::Server`].
pub(crate) fn response_outcome(message: &Value) -> Result<Value, LspError> {
    match message.get("error") {
        Some(err) => Err(LspError::Server {
            code: err.get("code").and_then(Value::as_i64).unwrap_or_default(),
            message: err
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
        }),
        None => Ok(message.get("result").cloned().unwrap_or(Value::Null)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_and_error_responses_map_to_outcomes() {
        let ok: Result<Value, LspError> = response_outcome(&json!({"id": 1, "result": {"a": 1}}));
        assert_eq!(ok.unwrap(), json!({"a": 1}));
        let null: Result<Value, LspError> = response_outcome(&json!({"id": 2}));
        assert_eq!(null.unwrap(), Value::Null);
        let err: Result<Value, LspError> =
            response_outcome(&json!({"id": 3, "error": {"code": -32601, "message": "nope"}}));
        assert!(matches!(err, Err(LspError::Server { code: -32601, .. })));
    }

    #[test]
    fn configuration_reply_answers_each_item_in_order() {
        let mut settings: Settings = Settings::new();
        settings.insert("a.known".to_owned(), json!(false));
        let params: Value = json!({"items": [{"section": "a.known"}, {"section": "a.other"}, {}]});
        assert_eq!(
            configuration_reply(Some(&params), &settings),
            json!([false, null, null])
        );
        assert_eq!(configuration_reply(None, &settings), json!([]));
    }
}
