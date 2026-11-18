//! `Content-Length` framing. LSP and DAP share this base protocol:
//! `Content-Length: N\r\n\r\n<N bytes of JSON>`.
//!
//! The [`Decoder`] is incremental: push bytes as they arrive, pull complete
//! messages out. A malformed body is reported and skipped so one bad frame
//! cannot wedge the connection.

use serde_json::Value;

/// Framing errors.
#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    /// Header bytes were not UTF-8.
    #[error("header is not valid UTF-8")]
    HeaderNotUtf8,
    /// No `Content-Length` header before the blank line.
    #[error("missing Content-Length header")]
    MissingContentLength,
    /// `Content-Length` was not a number.
    #[error("invalid Content-Length value: {0}")]
    InvalidContentLength(String),
    /// Body bytes were not JSON. The frame has already been discarded.
    #[error("body is not valid JSON: {0}")]
    BodyNotJson(#[from] serde_json::Error),
}

const HEADER_END: &[u8] = b"\r\n\r\n";

/// Encode one message with its `Content-Length` header.
#[must_use]
pub fn encode(message: &Value) -> Vec<u8> {
    let body: String = message.to_string();
    let mut out: Vec<u8> = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    out.extend_from_slice(body.as_bytes());
    out
}

/// Incremental frame decoder.
#[derive(Debug, Default)]
pub struct Decoder {
    buf: Vec<u8>,
}

impl Decoder {
    /// Append bytes read from the transport.
    pub fn push(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    /// Bytes buffered but not yet decoded.
    #[must_use]
    pub fn pending_len(&self) -> usize {
        self.buf.len()
    }

    /// The next complete message, or `Ok(None)` when more bytes are needed.
    ///
    /// # Errors
    /// A bad header, or a body that is not JSON (that frame is dropped first).
    pub fn next_message(&mut self) -> Result<Option<Value>, FrameError> {
        let Some(header_end): Option<usize> = find(&self.buf, HEADER_END) else {
            return Ok(None);
        };
        let header_bytes: &[u8] = self
            .buf
            .get(..header_end)
            .ok_or(FrameError::MissingContentLength)?;
        let header: &str = std::str::from_utf8(header_bytes)
            .map_err(|_: std::str::Utf8Error| FrameError::HeaderNotUtf8)?;
        let length: usize = content_length(header)?;
        let body_start: usize = header_end
            .checked_add(HEADER_END.len())
            .ok_or_else(overflow)?;
        let body_end: usize = body_start.checked_add(length).ok_or_else(overflow)?;
        let Some(body): Option<&[u8]> = self.buf.get(body_start..body_end) else {
            return Ok(None);
        };
        let parsed: Result<Value, serde_json::Error> = serde_json::from_slice::<Value>(body);
        self.buf.drain(..body_end);
        Ok(Some(parsed?))
    }
}

fn overflow() -> FrameError {
    FrameError::InvalidContentLength("overflow".to_owned())
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|w: &[u8]| w == needle)
}

fn content_length(header: &str) -> Result<usize, FrameError> {
    for line in header.split("\r\n") {
        let Some((name, value)): Option<(&str, &str)> = line.split_once(':') else {
            continue;
        };
        if name.trim().eq_ignore_ascii_case("content-length") {
            return value
                .trim()
                .parse::<usize>()
                .map_err(|e: std::num::ParseIntError| {
                    FrameError::InvalidContentLength(e.to_string())
                });
        }
    }
    Err(FrameError::MissingContentLength)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde_json::json;

    #[test]
    fn decodes_two_messages_from_one_chunk() {
        let mut d: Decoder = Decoder::default();
        let mut bytes: Vec<u8> = encode(&json!({"id": 1}));
        bytes.extend(encode(&json!({"id": 2})));
        d.push(&bytes);
        assert_eq!(d.next_message().unwrap(), Some(json!({"id": 1})));
        assert_eq!(d.next_message().unwrap(), Some(json!({"id": 2})));
        assert_eq!(d.next_message().unwrap(), None);
        assert_eq!(d.pending_len(), 0);
    }

    #[test]
    fn bad_json_body_is_reported_and_skipped() {
        let mut d: Decoder = Decoder::default();
        d.push(b"Content-Length: 3\r\n\r\n{{{");
        d.push(&encode(&json!("ok")));
        assert!(matches!(d.next_message(), Err(FrameError::BodyNotJson(_))));
        assert_eq!(d.next_message().unwrap(), Some(json!("ok")));
    }

    #[test]
    fn extra_headers_and_case_are_tolerated() {
        let mut d: Decoder = Decoder::default();
        d.push(b"content-type: application/json\r\ncontent-length: 2\r\n\r\n{}");
        assert_eq!(d.next_message().unwrap(), Some(json!({})));
    }

    proptest! {
        #[test]
        fn roundtrip_survives_any_split_point(text in ".{0,64}", split in 0usize..200) {
            let message: Value = json!({"method": text, "n": 7});
            let bytes: Vec<u8> = encode(&message);
            let cut: usize = split.min(bytes.len());
            let (head, tail): (&[u8], &[u8]) = bytes.split_at(cut);
            let mut d: Decoder = Decoder::default();
            d.push(head);
            let early: Option<Value> = d.next_message().unwrap();
            d.push(tail);
            let late: Option<Value> = d.next_message().unwrap();
            prop_assert!(early.is_none() || late.is_none());
            prop_assert_eq!(early.or(late), Some(message));
        }
    }
}
