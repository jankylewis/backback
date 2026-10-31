use crate::CharIdx;

use super::{Document, DocumentError, TextChange, UndoEdit};

impl Document {
    /// Insert text over the selection or at the cursor.
    ///
    /// # Errors
    /// Returns a checked text error on an invalid range.
    pub fn insert(&mut self, text: &str) -> Result<(), DocumentError> {
        let (start, end): (CharIdx, CharIdx) =
            self.selection().unwrap_or((self.cursor, self.cursor));
        let removed: String = self
            .buffer
            .slice(start, end)
            .ok_or(DocumentError::Selection)?;
        self.last_change = Some(TextChange {
            at: start,
            removed_chars: removed.chars().count(),
            inserted_bytes: text.len(),
        });
        let before: CharIdx = self.cursor;
        self.buffer.remove(start, end)?;
        self.buffer.insert(start, text)?;
        let after: CharIdx = CharIdx::new(start.raw().saturating_add(text.chars().count()));
        let can_group: bool = removed.is_empty()
            && text.chars().count() == 1
            && text.chars().all(char::is_alphanumeric)
            && self.word_group_open;
        if can_group && self.extend_word_group(start, text, after) {
            self.cursor = after;
        } else {
            self.undo.push(UndoEdit {
                start,
                removed,
                inserted: text.to_owned(),
                cursor_before: before,
                cursor_after: after,
            });
            self.cursor = after;
        }
        self.anchor = None;
        self.redo.clear();
        self.word_group_open = text.chars().count() == 1 && text.chars().all(char::is_alphanumeric);
        self.dirty = true;
        Ok(())
    }

    fn extend_word_group(&mut self, start: CharIdx, text: &str, after: CharIdx) -> bool {
        let Some(last): Option<&mut UndoEdit> = self.undo.last_mut() else {
            return false;
        };
        if last.cursor_after != start || !last.removed.is_empty() {
            return false;
        }
        last.inserted.push_str(text);
        last.cursor_after = after;
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
        let removed: String = self
            .buffer
            .slice(start, end)
            .ok_or(DocumentError::Selection)?;
        self.last_change = Some(TextChange {
            at: start,
            removed_chars: removed.chars().count(),
            inserted_bytes: 0,
        });
        let before: CharIdx = self.cursor;
        self.buffer.remove(start, end)?;
        self.undo.push(UndoEdit {
            start,
            removed,
            inserted: String::new(),
            cursor_before: before,
            cursor_after: start,
        });
        self.cursor = start;
        self.anchor = None;
        self.redo.clear();
        self.word_group_open = false;
        self.dirty = true;
        Ok(())
    }

    /// Insert a line break with the current line's indentation.
    ///
    /// # Errors
    /// Returns a checked text error on an invalid range.
    pub fn enter(&mut self) -> Result<(), DocumentError> {
        let text: String = self.buffer.to_text();
        let before: String = self
            .buffer
            .slice(CharIdx::new(0), self.cursor)
            .ok_or(DocumentError::Selection)?;
        let line: &str = before.rsplit(['\n', '\r']).next().unwrap_or("");
        let indent: String = line
            .chars()
            .take_while(|c: &char| *c == ' ' || *c == '\t')
            .collect();
        let ending: &str = if text.contains("\r\n") { "\r\n" } else { "\n" };
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
        self.replace_history(&edit, true)?;
        self.redo.push(edit);
        self.refresh_dirty();
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
        self.replace_history(&edit, false)?;
        self.undo.push(edit);
        self.refresh_dirty();
        Ok(())
    }

    fn replace_history(&mut self, edit: &UndoEdit, reverse: bool) -> Result<(), DocumentError> {
        let (old, new, cursor): (&str, &str, CharIdx) = if reverse {
            (&edit.inserted, &edit.removed, edit.cursor_before)
        } else {
            (&edit.removed, &edit.inserted, edit.cursor_after)
        };
        let end: CharIdx = CharIdx::new(edit.start.raw().saturating_add(old.chars().count()));
        self.buffer.remove(edit.start, end)?;
        self.buffer.insert(edit.start, new)?;
        self.last_change = Some(TextChange {
            at: edit.start,
            removed_chars: old.chars().count(),
            inserted_bytes: new.len(),
        });
        self.cursor = cursor;
        self.anchor = None;
        self.word_group_open = false;
        Ok(())
    }
}
