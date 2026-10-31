use std::collections::VecDeque;
use std::process::Stdio;
use std::time::Duration;

use bb_dap::Message;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

pub struct Session {
    child: Child,
    input: ChildStdin,
    output: ChildStdout,
    decoder: bb_framing::Decoder,
    events: VecDeque<Message>,
    seq: u64,
}

impl Session {
    pub fn start(binary: &str) -> Self {
        let mut child: Child = Command::new(binary)
            .arg("--interpreter=vscode")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let input: ChildStdin = child.stdin.take().unwrap();
        let output: ChildStdout = child.stdout.take().unwrap();
        Self {
            child,
            input,
            output,
            decoder: bb_framing::Decoder::default(),
            events: VecDeque::new(),
            seq: 0,
        }
    }

    async fn read(&mut self) -> Message {
        tokio::time::timeout(Duration::from_secs(15), async {
            loop {
                let value: Option<Value> = self.decoder.next_message().unwrap();
                if let Some(value) = value {
                    return Message::decode(value).unwrap();
                }
                let mut bytes: [u8; 8192] = [0; 8192];
                let count: usize = self.output.read(&mut bytes).await.unwrap();
                assert!(count > 0, "adapter closed output");
                self.decoder.push(&bytes[..count]);
            }
        })
        .await
        .unwrap()
    }

    pub async fn request(&mut self, command: &str, arguments: Value) -> (bool, Value) {
        self.seq += 1;
        let request: Message = Message::Request {
            seq: self.seq,
            command: command.to_owned(),
            arguments: Some(arguments),
        };
        self.input
            .write_all(&request.encode().unwrap())
            .await
            .unwrap();
        loop {
            let message: Message = self.read().await;
            match message {
                Message::Response {
                    request_seq,
                    success,
                    body,
                    ..
                } if request_seq == self.seq => {
                    return (success, body.unwrap_or(Value::Null));
                }
                Message::Event { .. } => self.events.push_back(message),
                _ => panic!("unexpected response"),
            }
        }
    }

    pub async fn event(&mut self, name: &str, reason: Option<&str>) -> Value {
        loop {
            let index: Option<usize> = self.events.iter().position(|message: &Message| {
                matches!(message, Message::Event { event, body, .. }
                    if event == name && reason.is_none_or(|expected: &str|
                        body.as_ref().and_then(|value: &Value| value.get("reason"))
                            .and_then(Value::as_str) == Some(expected)))
            });
            if let Some(index) = index {
                let message: Message = self.events.remove(index).unwrap();
                if let Message::Event { body, .. } = message {
                    return body.unwrap_or(Value::Null);
                }
            }
            let message: Message = self.read().await;
            self.events.push_back(message);
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if let Err(error) = self.child.start_kill() {
            eprintln!("adapter kill failed: {error}");
        }
    }
}
