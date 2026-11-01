//! Owned MCP stdio sessions with bounded requests and typed results.

mod bb_process;
mod session;
mod types;
mod wire;

pub use bb_process::{end_all_servers, exit_on_signal};
pub use session::Session;
pub use types::{CallResult, McpError, Resource, Tool};
