//! Decode adapter frames and route responses to the matching waiter.
//!
//! The reader does not decide liveness: the exit watcher aborts it when the
//! adapter process exits, even while a debuggee keeps the pipe open.

use serde_json::Value;
use tokio::io::AsyncReadExt;
use tokio::process::ChildStdout;
use tokio::sync::mpsc;

use crate::{DapError, Event, Message};

use super::lifecycle::Liveness;
use super::{Pending, Reply};

/// Prefix netcoredbg writes to stderr output before a condition-error stop.
const CONDITION_ERROR: &str = "Breakpoint error: ";

pub(super) async fn run(mut stdout: ChildStdout, live: Liveness, events: mpsc::Sender<Event>) {
    let mut decoder: bb_framing::Decoder = bb_framing::Decoder::default();
    let mut bytes: [u8; 8192] = [0; 8192];
    let mut pairing: Pairing = Pairing::default();
    loop {
        let count: usize = match stdout.read(&mut bytes).await {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        if let Some(chunk) = bytes.get(..count) {
            decoder.push(chunk);
        }
        loop {
            let value: Value = match decoder.next_message() {
                Ok(Some(value)) => value,
                Ok(None) => break,
                Err(_) => return live.close_all(),
            };
            let message: Message = match Message::decode(value) {
                Ok(message) => message,
                Err(_) => return live.close_all(),
            };
            match message {
                response @ Message::Response { .. } => handle_response(&live.pending, response),
                Message::Event { event, body, .. } => {
                    let event: Event = pairing.classify(Event::from_parts(event, body));
                    if matches!(event, Event::Terminated(_)) {
                        live.saw_terminated();
                    }
                    if events.send(event).await.is_err() {
                        return live.close_all();
                    }
                }
                Message::Request { .. } => {}
            }
        }
    }
    live.close_all();
}

/// Pairs a "Breakpoint error" output with the breakpoint stop that follows it.
#[derive(Default)]
struct Pairing {
    condition: Option<String>,
}

impl Pairing {
    fn classify(&mut self, event: Event) -> Event {
        match event {
            Event::Output(body) => {
                let text: Option<&str> = body.get("output").and_then(Value::as_str);
                if let Some(message) = text.filter(|t: &&str| t.starts_with(CONDITION_ERROR)) {
                    self.condition = Some(message.trim_end().to_owned());
                }
                Event::Output(body)
            }
            Event::Stopped(body) => {
                let reason: Option<&str> = body.get("reason").and_then(Value::as_str);
                match (self.condition.take(), reason) {
                    (Some(message), Some("breakpoint")) => Event::ConditionError {
                        message,
                        stopped: body,
                    },
                    (Some(_) | None, _) => Event::Stopped(body),
                }
            }
            event @ (Event::Terminated(_)
            | Event::Exited(_)
            | Event::Breakpoint(_)
            | Event::ConditionError { .. }
            | Event::Other(_, _)) => event,
        }
    }
}

fn handle_response(pending: &Pending, response: Message) {
    let (request_seq, success, command, message, body): (
        u64,
        bool,
        String,
        Option<String>,
        Option<Value>,
    ) = match response {
        Message::Response {
            request_seq,
            success,
            command,
            message,
            body,
            ..
        } => (request_seq, success, command, message, body),
        Message::Request { .. } | Message::Event { .. } => return,
    };
    let waiter: Option<tokio::sync::oneshot::Sender<Reply>> = pending.lock().ok().and_then(
        |mut map: std::sync::MutexGuard<
            '_,
            std::collections::HashMap<u64, tokio::sync::oneshot::Sender<Reply>>,
        >| map.remove(&request_seq),
    );
    if let Some(tx) = waiter {
        let result: Reply = if success {
            Ok(body.unwrap_or(Value::Null))
        } else {
            Err(DapError::Rejected {
                command,
                message: message.unwrap_or_else(|| "unknown adapter error".to_owned()),
            })
        };
        let _sent: Result<(), Reply> = tx.send(result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn stop(reason: &str) -> Event {
        Event::Stopped(json!({"reason": reason, "threadId": 1}))
    }

    #[test]
    fn breakpoint_stop_after_condition_error_output_is_a_condition_error() {
        let mut pairing: Pairing = Pairing::default();
        let text: &str = "Breakpoint error: The name 'nosuchvar' does not exist - /a/L.cs:7\n";
        let output: Event = pairing.classify(Event::Output(json!({"output": text})));
        assert!(matches!(output, Event::Output(_)));
        let paired: Event = pairing.classify(stop("breakpoint"));
        assert!(
            matches!(&paired, Event::ConditionError { message, .. } if message.contains("nosuchvar")),
            "{paired:?}"
        );
        assert_eq!(pairing.classify(stop("breakpoint")), stop("breakpoint"));
    }

    #[test]
    fn plain_output_and_other_stops_stay_as_they_are() {
        let mut pairing: Pairing = Pairing::default();
        pairing.classify(Event::Output(json!({"output": "hello\n"})));
        assert_eq!(pairing.classify(stop("breakpoint")), stop("breakpoint"));
        pairing.classify(Event::Output(
            json!({"output": "Breakpoint error: x - y:1\n"}),
        ));
        assert_eq!(pairing.classify(stop("step")), stop("step"));
        assert_eq!(pairing.classify(stop("breakpoint")), stop("breakpoint"));
    }
}
