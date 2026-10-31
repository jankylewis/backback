//! The one place that changes the text, and the two change logs it writes:
//! byte points for the incremental parser, char ranges for the language
//! server. Both logs see every edit, undo and redo, in the same order.

use crate::CharIdx;

use super::super::{Document, DocumentError, UndoEdit};

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

/// One edit as a language server mirror needs it: the chars `[from, to)` of
/// the text before it were replaced with `text`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharEdit {
    /// First char replaced.
    pub from: CharIdx,
    /// Char after the last one replaced, in the text before the edit.
    pub to: CharIdx,
    /// The new text.
    pub text: String,
}

/// Both change logs, drained by their own readers.
#[derive(Debug, Default)]
pub(in crate::document) struct ChangeLog {
    pub(in crate::document) bytes: Vec<TextChange>,
    pub(in crate::document) chars: Vec<CharEdit>,
}

impl Document {
    /// Char edits since the last call, oldest first. Applied in order to the
    /// text as it was at the previous call, they give the current text.
    pub fn take_char_edits(&mut self) -> Vec<CharEdit> {
        std::mem::take(&mut self.changes.chars)
    }

    /// Replace several ranges of the current text at once, as one undo step
    /// (a rename or a format). Ranges are in today's coordinates and must not
    /// overlap. Nothing changes when any range is bad.
    ///
    /// # Errors
    /// [`DocumentError::Selection`] when a range is reversed, overlaps
    /// another, or passes the end of the text.
    pub fn replace_many(&mut self, edits: &[CharEdit]) -> Result<(), DocumentError> {
        let mut sorted: Vec<&CharEdit> = edits.iter().collect();
        sorted.sort_by_key(|e: &&CharEdit| (e.from, e.to));
        let (Some(first), Some(last)): (Option<&&CharEdit>, Option<&&CharEdit>) =
            (sorted.first(), sorted.last())
        else {
            return Ok(());
        };
        let end: CharIdx = sorted
            .iter()
            .map(|e: &&CharEdit| e.to)
            .max()
            .unwrap_or(last.to);
        let start: CharIdx = first.from;
        let replaced: String = self.covered_text(&sorted, start, end)?;
        let before: CharIdx = self.cursor;
        let removed: String = self.splice(start, end, &replaced)?;
        let state_before: u64 = self.state;
        self.state = self.fresh_state();
        let after: CharIdx = shift_cursor(before, start, end, &replaced, &removed);
        self.undo.push(UndoEdit {
            start,
            removed,
            inserted: replaced,
            cursor_before: before,
            cursor_after: after,
            state_before,
            state_after: self.state,
        });
        self.cursor = after;
        self.anchor = None;
        self.redo.clear();
        self.word_group_open = false;
        Ok(())
    }

    /// The new text of `[start, end)` once every sorted edit is applied.
    fn covered_text(
        &self,
        sorted: &[&CharEdit],
        start: CharIdx,
        end: CharIdx,
    ) -> Result<String, DocumentError> {
        let mut out: String = String::new();
        let mut at: CharIdx = start;
        for edit in sorted {
            if edit.from < at || edit.to < edit.from || edit.to > end {
                return Err(DocumentError::Selection);
            }
            let kept: String = self
                .buffer
                .slice(at, edit.from)
                .ok_or(DocumentError::Selection)?;
            out.push_str(&kept);
            out.push_str(&edit.text);
            at = edit.to;
        }
        if end.raw() > self.len_chars() {
            return Err(DocumentError::Selection);
        }
        Ok(out)
    }

    /// The one place that changes the text: replace `[start, end)` with
    /// `text`, log the change in both logs and return the removed text.
    pub(in crate::document) fn splice(
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
        self.changes.bytes.push(TextChange {
            start: from,
            old_end,
            new_end,
        });
        self.changes.chars.push(CharEdit {
            from: start,
            to: end,
            text: text.to_owned(),
        });
        Ok(removed)
    }

    pub(in crate::document) fn byte_point(&self, at: CharIdx) -> Result<BytePoint, DocumentError> {
        let (byte, row, column): (usize, usize, usize) = self
            .buffer
            .byte_point(at, !self.format.other_breaks)
            .ok_or(DocumentError::Selection)?;
        Ok(BytePoint { byte, row, column })
    }
}

/// Where the cursor goes when `[start, end)` became `inserted`: before the
/// range it stays, after it moves with the length change, inside it goes to
/// the end of the new text.
fn shift_cursor(
    cursor: CharIdx,
    start: CharIdx,
    end: CharIdx,
    inserted: &str,
    removed: &str,
) -> CharIdx {
    if cursor <= start {
        return cursor;
    }
    let new_end: CharIdx = after_text(start, inserted);
    if cursor < end {
        return new_end;
    }
    let grown: usize = cursor
        .raw()
        .saturating_sub(removed.chars().count())
        .saturating_add(inserted.chars().count());
    CharIdx::new(grown)
}

pub(in crate::document) fn after_text(start: CharIdx, text: &str) -> CharIdx {
    CharIdx::new(start.raw().saturating_add(text.chars().count()))
}
