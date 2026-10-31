use crate::CharIdx;

use super::{BytePoint, Document, DocumentError, TextChange, UndoEdit};

impl Document {
    /// Insert text over the selection or at the cursor.
    ///
    /// # Errors
    /// Returns a checked text error on an invalid range.
    pub fn insert(&mut self, text: &str) -> Result<(), DocumentError> {
        let (start, end): (CharIdx, CharIdx) =
            self.selection().unwrap_or((self.cursor, self.cursor));
        let before: CharIdx = self.cursor;
        let removed: String = self.splice(start, end, text)?;
        let after: CharIdx = CharIdx::new(start.raw().saturating_add(text.chars().count()));
        let word_char: bool = text.chars().count() == 1 && text.chars().all(char::is_alphanumeric);
        let state_before: u64 = self.state;
        self.state = self.fresh_state();
        let grouped: bool = removed.is_empty()
            && word_char
            && self.word_group_open
            && self.extend_word_group(start, text, after);
        if !grouped {
            self.undo.push(UndoEdit {
                start,
                removed,
                inserted: text.to_owned(),
                cursor_before: before,
                cursor_after: after,
                state_before,
                state_after: self.state,
            });
        }
        self.cursor = after;
        self.anchor = None;
        self.redo.clear();
        self.word_group_open = word_char;
        Ok(())
    }

    fn extend_word_group(&mut self, start: CharIdx, text: &str, after: CharIdx) -> bool {
        let state: u64 = self.state;
        let Some(last): Option<&mut UndoEdit> = self.undo.last_mut() else {
            return false;
        };
        if last.cursor_after != start || !last.removed.is_empty() {
            return false;
        }
        last.inserted.push_str(text);
        last.cursor_after = after;
        last.state_after = state;
        true
    }

    /// Delete the selected chars or one char before the cursor.
    ///
    /// # Errors
    /// Returns a checked text error on an invalid range.
    pub fn backspace(&mut self) -> Result<(), DocumentError> {
        if let Some((start, end)) = self.selection() {
            self.delete_range(start, end)
        } else if self.cursor.raw() > 0 {
            self.delete_range(
                CharIdx::new(self.cursor.raw().saturating_sub(1)),
                self.cursor,
            )
        } else {
            Ok(())
        }
    }

    /// Delete the selected chars or one char after the cursor.
    ///
    /// # Errors
    /// Returns a checked text error on an invalid range.
    pub fn delete(&mut self) -> Result<(), DocumentError> {
        if let Some((start, end)) = self.selection() {
            self.delete_range(start, end)
        } else if self.cursor.raw() < self.len_chars() {
            self.delete_range(
                self.cursor,
                CharIdx::new(self.cursor.raw().saturating_add(1)),
            )
        } else {
            Ok(())
        }
    }

    fn delete_range(&mut self, start: CharIdx, end: CharIdx) -> Result<(), DocumentError> {
        let before: CharIdx = self.cursor;
        let removed: String = self.splice(start, end, "")?;
        let state_before: u64 = self.state;
        self.state = self.fresh_state();
        self.undo.push(UndoEdit {
            start,
            removed,
            inserted: String::new(),
            cursor_before: before,
            cursor_after: start,
            state_before,
            state_after: self.state,
        });
        self.cursor = start;
        self.anchor = None;
        self.redo.clear();
        self.word_group_open = false;
        Ok(())
    }

    /// Insert a line break with the cursor line's indentation (reads one line).
    ///
    /// # Errors
    /// Returns a checked text error on an invalid range.
    pub fn enter(&mut self) -> Result<(), DocumentError> {
        let line_start: CharIdx = self
            .line_start(self.cursor_line())
            .ok_or(DocumentError::Selection)?;
        let before: String = self
            .buffer
            .slice(line_start, self.cursor)
            .ok_or(DocumentError::Selection)?;
        let indent: String = before
            .chars()
            .take_while(|c: &char| *c == ' ' || *c == '\t')
            .collect();
        let ending: &str = if self.format.crlf { "\r\n" } else { "\n" };
        self.insert(&format!("{ending}{indent}"))
    }

    /// Undo the latest edit group.
    ///
    /// # Errors
    /// Returns a checked text error if the history is inconsistent.
    pub fn undo(&mut self) -> Result<(), DocumentError> {
        let Some(edit): Option<UndoEdit> = self.undo.pop() else {
            return Ok(());
        };
        let end: CharIdx = after_text(edit.start, &edit.inserted);
        self.splice(edit.start, end, &edit.removed)?;
        self.restore(edit.cursor_before, edit.state_before);
        self.redo.push(edit);
        Ok(())
    }

    /// Redo the latest undone edit group.
    ///
    /// # Errors
    /// Returns a checked text error if the history is inconsistent.
    pub fn redo(&mut self) -> Result<(), DocumentError> {
        let Some(edit): Option<UndoEdit> = self.redo.pop() else {
            return Ok(());
        };
        let end: CharIdx = after_text(edit.start, &edit.removed);
        self.splice(edit.start, end, &edit.inserted)?;
        self.restore(edit.cursor_after, edit.state_after);
        self.undo.push(edit);
        Ok(())
    }

    fn restore(&mut self, cursor: CharIdx, state: u64) {
        self.cursor = cursor;
        self.anchor = None;
        self.word_group_open = false;
        self.state = state;
    }

    /// The one place that changes the text: replace `[start, end)` with
    /// `text`, log the byte-level change and return the removed text.
    fn splice(
        &mut self,
        start: CharIdx,
        end: CharIdx,
        text: &str,
    ) -> Result<String, DocumentError> {
        let removed: String = self
            .buffer
            .slice(start, end)
            .ok_or(DocumentError::Selection)?;
        self.format.other_breaks =
            self.format.other_breaks || self.buffer.other_break_near(start, end);
        let from: BytePoint = self.byte_point(start)?;
        let old_end: BytePoint = self.byte_point(end)?;
        self.buffer.remove(start, end)?;
        self.buffer.insert(start, text)?;
        let end_after: CharIdx = after_text(start, text);
        self.format.other_breaks =
            self.format.other_breaks || self.buffer.other_break_near(start, end_after);
        let new_end: BytePoint = self.byte_point(end_after)?;
        self.changes.push(TextChange {
            start: from,
            old_end,
            new_end,
        });
        Ok(removed)
    }

    fn byte_point(&self, at: CharIdx) -> Result<BytePoint, DocumentError> {
        let (byte, row, column): (usize, usize, usize) = self
            .buffer
            .byte_point(at, !self.format.other_breaks)
            .ok_or(DocumentError::Selection)?;
        Ok(BytePoint { byte, row, column })
    }
}

fn after_text(start: CharIdx, text: &str) -> CharIdx {
    CharIdx::new(start.raw().saturating_add(text.chars().count()))
}
