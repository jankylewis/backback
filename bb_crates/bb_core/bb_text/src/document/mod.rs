//! File-backed editor state. Char indices are checked by the rope.

mod edit;
mod io;
mod movement;

use std::path::PathBuf;

use crate::{Buffer, CharIdx, TextError};

/// A document operation failed.
#[derive(Debug, thiserror::Error)]
pub enum DocumentError {
    /// No file was assigned to this buffer.
    #[error("document has no path")]
    NoPath,
    /// File I/O failed.
    #[error("file I/O: {0}")]
    Io(#[from] std::io::Error),
    /// File bytes are not UTF-8.
    #[error("file is not UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    /// A char index is invalid.
    #[error("text edit: {0}")]
    Text(#[from] TextError),
    /// A selection is outside the text.
    #[error("selection is outside the text")]
    Selection,
}

#[derive(Debug, Clone)]
struct UndoEdit {
    start: CharIdx,
    removed: String,
    inserted: String,
    cursor_before: CharIdx,
    cursor_after: CharIdx,
}

/// The most recent edit, in char units and byte lengths.
#[derive(Debug, Clone, Copy)]
pub struct TextChange {
    /// First changed char.
    pub at: CharIdx,
    /// Chars removed from the previous text.
    pub removed_chars: usize,
    /// Bytes inserted into the new text.
    pub inserted_bytes: usize,
}

/// One open text file and its undo history.
#[derive(Debug)]
pub struct Document {
    path: Option<PathBuf>,
    buffer: Buffer,
    saved: Buffer,
    cursor: CharIdx,
    anchor: Option<CharIdx>,
    undo: Vec<UndoEdit>,
    redo: Vec<UndoEdit>,
    bom: bool,
    dirty: bool,
    word_group_open: bool,
    last_change: Option<TextChange>,
}

impl Document {
    /// Create a buffer with optional backing path.
    #[must_use]
    pub fn from_text(text: &str, path: Option<PathBuf>) -> Self {
        let buffer: Buffer = Buffer::from_text(text);
        Self {
            path,
            saved: buffer.clone(),
            buffer,
            cursor: CharIdx::new(0),
            anchor: None,
            undo: Vec::new(),
            redo: Vec::new(),
            bom: false,
            dirty: false,
            word_group_open: false,
            last_change: None,
        }
    }

    /// The backing file path.
    #[must_use]
    pub fn path(&self) -> Option<&std::path::Path> {
        self.path.as_deref()
    }

    /// Full text for LSP or tests.
    #[must_use]
    pub fn text(&self) -> String {
        self.buffer.to_text()
    }

    /// Whether edits differ from the last saved content.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Current cursor.
    #[must_use]
    pub fn cursor(&self) -> CharIdx {
        self.cursor
    }

    /// Current selection, if nonempty.
    #[must_use]
    pub fn selection(&self) -> Option<(CharIdx, CharIdx)> {
        self.anchor
            .filter(|anchor: &CharIdx| *anchor != self.cursor)
            .map(|anchor: CharIdx| (anchor.min(self.cursor), anchor.max(self.cursor)))
    }

    /// Number of chars.
    #[must_use]
    pub fn len_chars(&self) -> usize {
        self.buffer.len_chars()
    }

    /// Number of lines.
    #[must_use]
    pub fn len_lines(&self) -> usize {
        self.buffer.len_lines()
    }

    /// A line with its existing line break.
    #[must_use]
    pub fn line(&self, index: usize) -> Option<String> {
        self.buffer.line(index)
    }

    /// Start char index of a line.
    #[must_use]
    pub fn line_start(&self, index: usize) -> Option<CharIdx> {
        self.buffer.line_start_char(index)
    }

    /// Change that produced the current text.
    #[must_use]
    pub fn last_change(&self) -> Option<TextChange> {
        self.last_change
    }

    /// Move the cursor and clear the selection.
    ///
    /// # Errors
    /// Returns [`DocumentError::Selection`] if the index is out of bounds.
    pub fn set_cursor(&mut self, at: CharIdx) -> Result<(), DocumentError> {
        self.check(at)?;
        self.cursor = at;
        self.anchor = None;
        self.word_group_open = false;
        Ok(())
    }

    /// Set a selection from `anchor` to `cursor`.
    ///
    /// # Errors
    /// Returns [`DocumentError::Selection`] if either index is out of bounds.
    pub fn select(&mut self, anchor: CharIdx, cursor: CharIdx) -> Result<(), DocumentError> {
        self.check(anchor)?;
        self.check(cursor)?;
        self.anchor = Some(anchor);
        self.cursor = cursor;
        self.word_group_open = false;
        Ok(())
    }

    fn check(&self, at: CharIdx) -> Result<(), DocumentError> {
        if at.raw() > self.buffer.len_chars() {
            Err(DocumentError::Selection)
        } else {
            Ok(())
        }
    }

    fn refresh_dirty(&mut self) {
        self.dirty = self.buffer.to_text() != self.saved.to_text();
    }
}

#[cfg(test)]
mod bb_tests;
