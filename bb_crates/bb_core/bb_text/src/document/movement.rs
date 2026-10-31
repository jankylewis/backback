use super::{Document, DocumentError};
use crate::CharIdx;

impl Document {
    /// Line containing the cursor.
    #[must_use]
    pub fn cursor_line(&self) -> usize {
        self.buffer.line_of_char(self.cursor).unwrap_or(0)
    }

    /// Move one char left, optionally extending the selection.
    ///
    /// # Errors
    /// Returns a checked selection error.
    pub fn left(&mut self, extend: bool) -> Result<(), DocumentError> {
        self.move_to(CharIdx::new(self.cursor.raw().saturating_sub(1)), extend)
    }

    /// Move one char right, optionally extending the selection.
    ///
    /// # Errors
    /// Returns a checked selection error.
    pub fn right(&mut self, extend: bool) -> Result<(), DocumentError> {
        self.move_to(
            CharIdx::new(self.cursor.raw().saturating_add(1).min(self.len_chars())),
            extend,
        )
    }

    /// Move one line while keeping the char column when it exists.
    ///
    /// # Errors
    /// Returns a checked selection error.
    pub fn vertical(&mut self, down: bool, extend: bool) -> Result<(), DocumentError> {
        let line: usize = self
            .buffer
            .line_of_char(self.cursor)
            .ok_or(DocumentError::Selection)?;
        let start: usize = self.line_start(line).ok_or(DocumentError::Selection)?.raw();
        let column: usize = self.cursor.raw().saturating_sub(start);
        let target_line: usize = if down {
            line.saturating_add(1)
                .min(self.len_lines().saturating_sub(1))
        } else {
            line.saturating_sub(1)
        };
        let target_start: usize = self
            .line_start(target_line)
            .ok_or(DocumentError::Selection)?
            .raw();
        let target_text: String = self.line(target_line).ok_or(DocumentError::Selection)?;
        let visible_chars: usize = target_text.trim_end_matches(['\r', '\n']).chars().count();
        let target: CharIdx = CharIdx::new(target_start.saturating_add(column.min(visible_chars)));
        self.move_to(target, extend)
    }

    /// Set cursor and optionally extend the old selection.
    ///
    /// # Errors
    /// Returns a checked selection error.
    pub fn move_to(&mut self, target: CharIdx, extend: bool) -> Result<(), DocumentError> {
        self.check(target)?;
        if extend && self.anchor.is_none() {
            self.anchor = Some(self.cursor);
        } else if !extend {
            self.anchor = None;
        }
        self.cursor = target;
        self.word_group_open = false;
        Ok(())
    }
}
