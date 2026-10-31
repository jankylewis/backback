//! Decode adapter frames and route responses to the matching waiter.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::process::ChildStdout;
use tokio::sync::mpsc;

use crate::{DapError, Event, Message};

use super::Pending;

pub(super) async fn run(
    mut stdout: ChildStdout,
    pending: Pending,
    closed: Arc<AtomicBool>,
    events: mpsc::Sender<Event>,
) {
    let mut decoder: bb_framing::Decoder = bb_framing::Decoder::default();
    let mut bytes: [u8; 8192] = [0; 8192];
    loop {
        let count: usize = match stdout.read(&mut bytes).await {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        if let Some(chunk) = bytes.get(..count) {
            decoder.push(chunk);
        }
        loop {
            let value: serde_json::Value = match decoder.next_message() {
                Ok(Some(value)) => value,
                Ok(None) => break,
                Err(_) => return close_all(&pending, &closed),
            };
            let message: Message = match Message::decode(value) {
                Ok(message) => message,
                Err(_) => return close_all(&pending, &closed),
            };
            match message {
                response @ Message::Response { .. } => handle_response(&pending, response),
                Message::Event { event, body, .. } => {
                    if events.send(Event::from_parts(event, body)).await.is_err() {
                        return close_all(&pending, &closed);
                    }
                }
                Message::Request { .. } => {}
            }
        }
    }
    close_all(&pending, &closed);
}

fn handle_response(pending: &Pending, response: Message) {
    let (request_seq, success, command, message, body): (
        u64,
        bool,
        String,
        Option<String>,
        Option<serde_json::Value>,
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
    let waiter: Option<tokio::sync::oneshot::Sender<super::Reply>> = pending.lock().ok().and_then(
        |mut map: std::sync::MutexGuard<
            '_,
            std::collections::HashMap<u64, tokio::sync::oneshot::Sender<super::Reply>>,
        >| map.remove(&request_seq),
    );
    if let Some(tx) = waiter {
        let result: super::Reply = if success {
            Ok(body.unwrap_or(serde_json::Value::Null))
        } else {
            Err(DapError::Rejected {
                command,
                message: message.unwrap_or_else(|| "unknown adapter error".to_owned()),
            })
        };
        let _sent: Result<(), super::Reply> = tx.send(result);
    }
}

fn close_all(pending: &Pending, closed: &AtomicBool) {
    closed.store(true, Ordering::Release);
    if let Ok(mut map) = pending.lock() {
        for (_, tx) in map.drain() {
            let _sent: Result<(), super::Reply> = tx.send(Err(DapError::Closed));
        }
    }
}
