//! Roslyn code intelligence for backback, behind a typed API.
//!
//! A [`Session`] starts `roslyn-language-server`, loads one solution and
//! keeps a mirror of every open document in a [`bb_text::Buffer`]. Edits go
//! to the server as incremental `didChange` events computed from that mirror,
//! with a version that only goes up. Every feature is one typed function with
//! its own deadline; nothing here applies a server edit to disk.

mod bb_session;
mod features;
mod model;
mod sync;

pub use bb_session::{Config, Session};
pub use model::bb_doc::{DocVersion, Edit};
pub use model::results::{
    ActionItem, CompletionEntry, Diagnostic, FileChange, HoverInfo, Severity, TextChange,
};
pub use model::{DocPath, DocUri, Location, Span};

use std::path::PathBuf;

/// Everything a [`Session`] call can fail with.
#[derive(Debug, thiserror::Error)]
pub enum IntelError {
    /// Starting, initializing or loading the server failed.
    #[error(transparent)]
    Roslyn(#[from] bb_lsp::roslyn::RoslynError),
    /// A request failed (timeout, closed, server error, bad shape).
    #[error(transparent)]
    Lsp(#[from] bb_lsp::LspError),
    /// The solution file cannot be read.
    #[error(transparent)]
    Solution(#[from] bb_dotnet::SolutionError),
    /// An edit or a position does not fit the document.
    #[error(transparent)]
    Text(#[from] bb_text::TextError),
    /// A path is not absolute, or cannot become a `file:` URI.
    #[error("not an absolute file path: {0}")]
    BadPath(PathBuf),
    /// A feature was asked for a document that is not open.
    #[error("document is not open: {0}")]
    NotOpen(PathBuf),
    /// The document is already open; close it first.
    #[error("document is already open: {0}")]
    AlreadyOpen(PathBuf),
    /// The document version cannot go higher.
    #[error("document version overflow: {0}")]
    VersionOverflow(PathBuf),
    /// The server's formatter failed on this document (Roslyn: `-32000` on
    /// some U+0085/U+2028/U+2029 trivia). The session keeps working.
    #[error("formatter failed on {path} (server error {code}): {message}")]
    Formatter {
        /// The document.
        path: PathBuf,
        /// JSON-RPC error code.
        code: i64,
        /// Server message.
        message: String,
    },
    /// A file could not be read.
    #[error("cannot read {path}: {source}")]
    Read {
        /// The file.
        path: PathBuf,
        /// The cause.
        #[source]
        source: std::io::Error,
    },
}
