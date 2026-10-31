//! Debug Adapter Protocol for backback.
//!
//! The debugger is `netcoredbg` (Samsung, MIT). Microsoft's `vsdbg` is licensed
//! for Visual Studio and VS Code only, so it is never used here. DAP shares the
//! `Content-Length` framing with LSP ([`bb_framing`]); this crate adds the DAP
//! message envelope and adapter discovery. The session state machine lands in M3.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Binary name of the debug adapter.
pub const BINARY: &str = "netcoredbg";
/// Environment variable that overrides discovery.
pub const ENV_OVERRIDE: &str = "BACKBACK_NETCOREDBG";
/// Arguments that put netcoredbg into DAP mode over stdio.
pub const DAP_ARGS: [&str; 1] = ["--interpreter=vscode"];

/// DAP errors.
#[derive(Debug, thiserror::Error)]
pub enum DapError {
    /// The adapter binary was not found.
    #[error(
        "{BINARY} not found; set {ENV_OVERRIDE} or install netcoredbg (MIT) and put it on PATH"
    )]
    NotFound,
    /// A message did not match the DAP envelope.
    #[error("not a DAP message: {0}")]
    Envelope(#[from] serde_json::Error),
}

/// The DAP envelope. `type` decides which of the optional fields are present.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Message {
    /// Client -> adapter.
    Request {
        /// Sequence number.
        seq: u64,
        /// Command name, e.g. `initialize`, `launch`, `setBreakpoints`.
        command: String,
        /// Command arguments.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        arguments: Option<Value>,
    },
    /// Adapter -> client, answering a request.
    Response {
        /// Sequence number.
        seq: u64,
        /// The `seq` of the request being answered.
        request_seq: u64,
        /// Whether the request succeeded.
        success: bool,
        /// Command name echoed back.
        command: String,
        /// Error text when `success` is false.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
        /// Result body.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        body: Option<Value>,
    },
    /// Adapter -> client, unsolicited (`stopped`, `output`, `terminated`, ...).
    Event {
        /// Sequence number.
        seq: u64,
        /// Event name.
        event: String,
        /// Event body.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        body: Option<Value>,
    },
}

impl Message {
    /// Encode with `Content-Length` framing.
    ///
    /// # Errors
    /// [`DapError::Envelope`] when the message cannot be serialised.
    pub fn encode(&self) -> Result<Vec<u8>, DapError> {
        Ok(bb_framing::encode(&serde_json::to_value(self)?))
    }

    /// Decode from a framed JSON value.
    ///
    /// # Errors
    /// [`DapError::Envelope`] when the value is not a DAP message.
    pub fn decode(value: Value) -> Result<Self, DapError> {
        Ok(serde_json::from_value(value)?)
    }
}

/// Find netcoredbg: env override, then `PATH`.
///
/// # Errors
/// [`DapError::NotFound`] when it is in neither place.
pub fn locate() -> Result<PathBuf, DapError> {
    if let Some(p) = std::env::var_os(ENV_OVERRIDE) {
        let p: PathBuf = PathBuf::from(p);
        return if p.is_file() {
            Ok(p)
        } else {
            Err(DapError::NotFound)
        };
    }
    let names: [String; 2] = [BINARY.to_owned(), format!("{BINARY}.exe")];
    std::env::var_os("PATH")
        .map(|p: std::ffi::OsString| std::env::split_paths(&p).collect::<Vec<PathBuf>>())
        .unwrap_or_default()
        .iter()
        .flat_map(|d: &PathBuf| names.iter().map(move |n: &String| d.join(n)))
        .find(|p: &PathBuf| p.is_file())
        .ok_or(DapError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn request_roundtrips_through_framing() {
        let req: Message = Message::Request {
            seq: 1,
            command: "initialize".to_owned(),
            arguments: Some(json!({ "adapterID": "backback" })),
        };
        let bytes: Vec<u8> = req.encode().unwrap();
        let mut d: bb_framing::Decoder = bb_framing::Decoder::default();
        d.push(&bytes);
        let value: Value = d.next_message().unwrap().unwrap();
        assert_eq!(Message::decode(value).unwrap(), req);
    }

    #[test]
    fn event_without_body_decodes() {
        let value: Value = json!({ "type": "event", "seq": 9, "event": "terminated" });
        assert_eq!(
            Message::decode(value).unwrap(),
            Message::Event {
                seq: 9,
                event: "terminated".to_owned(),
                body: None
            }
        );
    }
}
