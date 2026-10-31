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
    /// The target file cannot be written; it was left unchanged.
    #[error("file is read-only: {}", .0.display())]
    ReadOnly(PathBuf),
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
    state_before: u64,
    state_after: u64,
}

/// A byte offset with its 0-based row and byte column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BytePoint {
    /// Byte offset from the start of the text.
    pub byte: usize,
    /// 0-based line.
    pub row: usize,
    /// Bytes from the start of the line.
    pub column: usize,
}

/// One edit as an incremental parser needs it: where it starts, where the
/// removed text ended in the old text, and where the inserted text ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextChange {
    /// First changed byte (same in old and new text).
    pub start: BytePoint,
    /// End of the removed text, in the old text.
    pub old_end: BytePoint,
    /// End of the inserted text, in the new text.
    pub new_end: BytePoint,
}

/// How the file's bytes look, fixed at load (other breaks only ever turn on).
#[derive(Debug, Clone, Copy)]
struct TextFormat {
    /// The file starts with a UTF-8 BOM.
    bom: bool,
    /// New line breaks are CRLF.
    crlf: bool,
    /// The text has a line break besides LF/CRLF, so LF rows need a scan.
    other_breaks: bool,
}

/// One open text file and its undo history.
#[derive(Debug)]
pub struct Document {
    path: Option<PathBuf>,
    buffer: Buffer,
    cursor: CharIdx,
    anchor: Option<CharIdx>,
    undo: Vec<UndoEdit>,
    redo: Vec<UndoEdit>,
    format: TextFormat,
    word_group_open: bool,
    changes: Vec<TextChange>,
    state: u64,
    saved_state: u64,
    next_state: u64,
}

impl Document {
    /// Create a buffer with optional backing path.
    #[must_use]
    pub fn from_text(text: &str, path: Option<PathBuf>) -> Self {
        let buffer: Buffer = Buffer::from_text(text);
        let other_breaks: bool = Buffer::text_has_other_break(text);
        Self {
            path,
            buffer,
            format: TextFormat {
                bom: false,
                crlf: text.contains("\r\n"),
                other_breaks,
            },
            cursor: CharIdx::new(0),
            anchor: None,
            undo: Vec::new(),
            redo: Vec::new(),
            word_group_open: false,
            changes: Vec::new(),
            state: 0,
            saved_state: 0,
            next_state: 1,
        }
    }

    /// The backing file path.
    #[must_use]
    pub fn path(&self) -> Option<&std::path::Path> {
        self.path.as_deref()
    }

    /// The rope, for readers that walk the text in chunks (parsers, highlighters).
    #[must_use]
    pub fn rope(&self) -> &ropey::Rope {
        self.buffer.rope()
    }

    /// Full text for LSP or tests. Allocates; never call it per keystroke.
    #[must_use]
    pub fn text(&self) -> String {
        self.buffer.to_text()
    }

    /// Whether the text may differ from the last saved content.
    ///
    /// Each edit makes a new state; undo and redo return to earlier states, so
    /// undoing back to the save point is clean. O(1).
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.state != self.saved_state
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

    /// Start byte offset of a line.
    #[must_use]
    pub fn line_start_byte(&self, index: usize) -> Option<usize> {
        self.buffer.line_start_byte(index)
    }

    /// Edits since the last call, oldest first. The caller applies them in order.
    pub fn take_changes(&mut self) -> Vec<TextChange> {
        std::mem::take(&mut self.changes)
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

    fn fresh_state(&mut self) -> u64 {
        let state: u64 = self.next_state;
        self.next_state = self.next_state.saturating_add(1);
        state
    }
}

#[cfg(test)]
mod bb_tests;
