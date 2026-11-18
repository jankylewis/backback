//! A session with one language server: Roslyn (`bb_session`), or
//! rust-analyzer or pyright (`servers`), the document sync all share
//! (`sync`), and the pump that reads what the server sends on its own.

pub(crate) mod bb_session;
pub(crate) mod servers;
mod sync;

use std::sync::Arc;

use bb_lsp::{roslyn, Incoming};
use lsp_types::PublishDiagnosticsParams;
use tokio::sync::mpsc;
use tokio::sync::Notify;

use crate::model::bb_doc::DocVersion;
use crate::model::store::{Entry, Recorded, Store};
use crate::model::DocUri;

/// Which language server a session runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Server {
    /// `roslyn-language-server`, for C#.
    Roslyn,
    /// rust-analyzer, for Rust (BBEOD27).
    RustAnalyzer,
    /// pyright (or basedpyright), for Python (BBEOD28).
    Pyright,
}

impl Server {
    /// The `languageId` of the documents it is sent.
    #[must_use]
    pub fn language_id(self) -> &'static str {
        match self {
            Self::Roslyn => "csharp",
            Self::RustAnalyzer => "rust",
            Self::Pyright => "python",
        }
    }

    /// The name people read in the status bar.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Roslyn => "C# server",
            Self::RustAnalyzer => "Rust server",
            Self::Pyright => "Python server",
        }
    }

    /// Whether its diagnostics are pushed (`publishDiagnostics`), not
    /// pulled: rust-analyzer and pyright push, Roslyn answers pulls.
    #[must_use]
    pub fn pushes_diagnostics(self) -> bool {
        match self {
            Self::Roslyn => false,
            Self::RustAnalyzer | Self::Pyright => true,
        }
    }
}

/// Keep reading server messages so the reader never blocks on a full
/// channel; write each one to the run log, signal `refresh` when the lists
/// may be stale (for a server that pushes, after every push), and keep
/// each pushed `publishDiagnostics` per document unless a newer version is
/// stored.
pub(crate) async fn pump<W: std::io::Write>(
    mut incoming: mpsc::Receiver<Incoming>,
    (store, refresh): (Store, Arc<Notify>),
    server: Server,
    mut log: W,
) {
    while let Some(message) = incoming.recv().await {
        // A log that stops accepting lines must not stop the drain: the
        // reader would block and every later request would time out.
        let text: &str = roslyn::log_incoming(&message, &mut log).unwrap_or_default();
        if crate::features::diagnostics::refresh_requested(&message, text) {
            refresh.notify_one();
        }
        let Incoming::Notification { method, params }: Incoming = message else {
            continue;
        };
        if method != "textDocument/publishDiagnostics" {
            continue;
        }
        let Some(report): Option<PublishDiagnosticsParams> =
            params.and_then(|p: serde_json::Value| serde_json::from_value(p).ok())
        else {
            continue;
        };
        let DocUri::File(path): DocUri = DocUri::from_url(&report.uri) else {
            continue;
        };
        let entry: Entry = Entry {
            version: report.version.and_then(DocVersion::from_wire),
            list: report.diagnostics,
        };
        match store.record(&path, entry) {
            Ok(Recorded::Kept) if server.pushes_diagnostics() => refresh.notify_one(),
            Ok(Recorded::Kept | Recorded::Stale) => {}
            // Every reader now fails with the same error; the log says when.
            Err(error) => {
                let _logged: std::io::Result<()> = writeln!(log, "backback: {error}");
            }
        }
    }
}
