//! LSP client for backback.
//!
//! [`bb_client::LspClient`] is a generic JSON-RPC-over-stdio client: one writer
//! task, one reader task, responses matched by id, server notifications on a
//! channel, and a deadline on every request. [`roslyn`] adds what the official
//! `roslyn-language-server` needs beyond plain LSP (`solution/open`, the
//! `projectInitializationComplete` signal, its command-line flags).

pub mod bb_client;
pub mod roslyn;
mod wire;

pub use bb_client::{Incoming, LspClient, LspError, Settings};
