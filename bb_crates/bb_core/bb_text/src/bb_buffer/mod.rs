//! Rope-backed buffer. All edits and conversions are bounds-checked and return
//! a typed error; nothing here panics.

use ropey::Rope;

use crate::position::{CharIdx, LineIdx, LspPoint};

/// Errors from buffer edits and position conversions.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum TextError {
    /// A char index past the end of the buffer.
    #[error("char index {index} is out of bounds (len {len})")]
    CharOutOfBounds {
        /// The offending index.
        index: usize,
        /// The buffer length in chars.
        len: usize,
    },
    /// A line index past the last line.
    #[error("line {line} is out of bounds (lines {lines})")]
    LineOutOfBounds {
        /// The offending line.
        line: usize,
        /// The number of lines.
        lines: usize,
    },
    /// A UTF-16 column past the end of its line.
    #[error("utf-16 column {character} is past the end of line {line}")]
    ColumnOutOfBounds {
        /// The line.
        line: u32,
        /// The offending column.
        character: u32,
    },
    /// A value does not fit the `u32` that LSP uses.
    #[error("value {0} does not fit in u32")]
    TooLargeForLsp(usize),
}

/// A text buffer. Cheap to clone (the rope shares structure).
#[derive(Clone, Debug, Default)]
pub struct Buffer {
    rope: Rope,
}

impl Buffer {
    /// Build a buffer from text.
    #[must_use]
    pub fn from_text(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
        }
    }

    /// Number of chars.
    #[must_use]
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    /// Number of lines (a trailing line break counts one more empty line).
    #[must_use]
    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    /// The whole text. Allocates; use for tests, saves and LSP `didOpen`.
    #[must_use]
    pub fn to_text(&self) -> String {
        self.rope.to_string()
    }

    /// The char at `at`; `None` past the end.
    #[must_use]
    pub fn char_at(&self, at: CharIdx) -> Option<char> {
        self.rope.get_char(at.raw())
    }

    /// Insert `text` at `at`.
    ///
    /// # Errors
    /// [`TextError::CharOutOfBounds`] when `at` is past the end.
    pub fn insert(&mut self, at: CharIdx, text: &str) -> Result<(), TextError> {
        let oob: TextError = self.char_oob(at);
        self.rope
            .try_insert(at.raw(), text)
            .map_err(|_: ropey::Error| oob)
    }

    /// Remove the chars in `[from, to)`.
    ///
    /// # Errors
    /// [`TextError::CharOutOfBounds`] when the range is reversed or past the end.
    pub fn remove(&mut self, from: CharIdx, to: CharIdx) -> Result<(), TextError> {
        let oob: TextError = self.char_oob(if from > to { from } else { to });
        if from > to {
            return Err(oob);
        }
        self.rope
            .try_remove(from.raw()..to.raw())
            .map_err(|_: ropey::Error| oob)
    }

    /// Convert a char index to an LSP point (line + UTF-16 column).
    ///
    /// # Errors
    /// Out-of-bounds index, or a value too large for LSP's `u32`.
    pub fn to_lsp(&self, at: CharIdx) -> Result<LspPoint, TextError> {
        let line: usize = self.line_of(at)?;
        let line_start_cu: usize = self.utf16_at(self.line_start(line)?)?;
        let at_cu: usize = self.utf16_at(at)?;
        let column: usize = at_cu
            .checked_sub(line_start_cu)
            .ok_or_else(|| self.char_oob(at))?;
        Ok(LspPoint {
            line: to_u32(line)?,
            character: to_u32(column)?,
        })
    }

    /// Convert an LSP point to a char index. A column past the end of its line
    /// is clamped to the line end, as the LSP specification requires.
    ///
    /// # Errors
    /// [`TextError::LineOutOfBounds`] when the line does not exist.
    pub fn from_lsp(&self, point: LspPoint) -> Result<CharIdx, TextError> {
        let line: usize = from_u32(point.line);
        let start: CharIdx = self.line_start(line)?;
        let start_cu: usize = self.utf16_at(start)?;
        let column: usize = from_u32(point.character).min(self.line_len_cu(line));
        let target_cu: usize = start_cu.saturating_add(column);
        let oob: TextError = TextError::TooLargeForLsp(target_cu);
        let idx: usize = self
            .rope
            .try_utf16_cu_to_char(target_cu)
            .map_err(|_: ropey::Error| oob)?;
        Ok(CharIdx::new(idx))
    }

    /// UTF-16 length of `line` without its line break.
    fn line_len_cu(&self, line: usize) -> usize {
        let Some(slice): Option<ropey::RopeSlice<'_>> = self.rope.get_line(line) else {
            return 0;
        };
        let breaks: usize = match (
            char_from_end(slice, 2) == Some('\r'),
            char_from_end(slice, 1) == Some('\n'),
        ) {
            (true, true) => 2,
            (false, true) => 1,
            (true | false, false) => 0,
        };
        slice.len_utf16_cu().saturating_sub(breaks)
    }

    fn line_of(&self, at: CharIdx) -> Result<usize, TextError> {
        let oob: TextError = self.char_oob(at);
        self.rope
            .try_char_to_line(at.raw())
            .map_err(|_: ropey::Error| oob)
    }

    fn line_start(&self, line: usize) -> Result<CharIdx, TextError> {
        let oob: TextError = TextError::LineOutOfBounds {
            line: LineIdx::new(line).raw(),
            lines: self.rope.len_lines(),
        };
        let start: usize = self
            .rope
            .try_line_to_char(line)
            .map_err(|_: ropey::Error| oob)?;
        Ok(CharIdx::new(start))
    }

    fn utf16_at(&self, at: CharIdx) -> Result<usize, TextError> {
        let oob: TextError = self.char_oob(at);
        self.rope
            .try_char_to_utf16_cu(at.raw())
            .map_err(|_: ropey::Error| oob)
    }

    fn char_oob(&self, at: CharIdx) -> TextError {
        TextError::CharOutOfBounds {
            index: at.raw(),
            len: self.rope.len_chars(),
        }
    }
}

/// The char `back` positions before the end of `slice` (1 = last char).
fn char_from_end(slice: ropey::RopeSlice<'_>, back: usize) -> Option<char> {
    slice
        .len_chars()
        .checked_sub(back)
        .and_then(|i: usize| slice.get_char(i))
}

fn to_u32(value: usize) -> Result<u32, TextError> {
    u32::try_from(value).map_err(|_: std::num::TryFromIntError| TextError::TooLargeForLsp(value))
}

/// `u32` always fits `usize` on the 32/64-bit targets backback supports.
fn from_u32(value: u32) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod bb_tests;
