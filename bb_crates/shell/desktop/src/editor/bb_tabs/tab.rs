//! One open file: its document, its syntax tree, and whether it may change.

use std::path::PathBuf;
use std::time::Instant;

use bb_syntax::{CSharpSyntax, LiveTree, SyntaxError, Tree};
use bb_text::{Document, DocumentError, TextChange};

use super::window;

pub(crate) struct Tab {
    pub(crate) path: PathBuf,
    pub(crate) document: Document,
    /// Highlight queries; parsing uses the parser inside `live`.
    pub(crate) highlighter: CSharpSyntax,
    pub(crate) live: LiveTree,
    /// Lines the tree covers; `None` for a file parsed whole.
    pub(crate) window_lines: Option<std::ops::Range<usize>>,
    /// No edit, save or rename may change it (decompiled metadata, a file
    /// the user cannot write).
    pub(crate) read_only: bool,
}

/// A tab opened or closed; the code intelligence link reads these.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TabEvent {
    /// The tab for this file opened.
    Opened(PathBuf),
    /// The tab for this file closed.
    Closed(PathBuf),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum TabError {
    #[error("document: {0}")]
    Document(#[from] DocumentError),
    #[error("syntax: {0}")]
    Syntax(#[from] SyntaxError),
    #[error("file path: {0}")]
    Path(#[from] std::io::Error),
}

impl Tab {
    /// Read and parse a file. Slow on a big file: call it off the UI thread.
    pub(crate) fn load(path: PathBuf) -> Result<Self, TabError> {
        let started: Instant = Instant::now();
        let document: Document = Document::open(&path)?;
        let window_lines: Option<std::ops::Range<usize>> = window::first_window(&document);
        let mut syntax: CSharpSyntax = CSharpSyntax::new()?;
        let tree: Tree = syntax.parse_rope_in(
            document.rope(),
            None,
            window::points(&document, window_lines.as_ref()),
        )?;
        let highlighter: CSharpSyntax = CSharpSyntax::new()?;
        let read_only: bool = std::fs::metadata(&path)
            .is_ok_and(|meta: std::fs::Metadata| meta.permissions().readonly());
        eprintln!(
            "editor_open_ms={:.3}",
            started.elapsed().as_secs_f64() * 1000.0
        );
        let mut document: Document = document;
        if read_only {
            document.lock_writes();
        }
        Ok(Self {
            path,
            document,
            highlighter,
            live: LiveTree::new(syntax, tree),
            window_lines,
            read_only,
        })
    }

    /// Move the document's new edits onto the shown tree. O(depth) per edit;
    /// the parse itself runs later, off the UI thread.
    pub(crate) fn sync_edits(&mut self) {
        let changes: Vec<TextChange> = self.document.take_changes();
        self.live.edit(&changes);
    }

    /// Make the tab read-only: no edit, and no save from any path.
    pub(crate) fn set_read_only(&mut self) {
        self.read_only = true;
        self.document.lock_writes();
    }

    /// Sync and parse on this thread, as a test oracle for the background path.
    #[cfg(test)]
    pub(crate) fn reparse(&mut self) -> Result<(), TabError> {
        self.sync_edits();
        self.live
            .parse_now(self.document.rope(), self.parse_window())?;
        Ok(())
    }
}
