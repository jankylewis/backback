//! Line-wise edits (BB-BUG-014, BB-BUG-015): delete to a line's start or
//! end, and indent or outdent the lines a selection touches, as VS Code and
//! Rider do. The indent unit is four spaces, the C# default.

use super::bb_splice::CharEdit;
use super::{Document, DocumentError};
use crate::CharIdx;

/// One indent level: four spaces, the C# default of VS Code and Rider.
const INDENT: &str = "    ";
/// A line comment, as VS Code and Rider write it.
const COMMENT: &str = "// ";

impl Document {
    /// Cmd+Backspace: the selection, else the line from its start to the
    /// cursor; at a line start, the line break before it (as VS Code).
    ///
    /// # Errors
    /// Returns a checked text error.
    pub fn delete_to_line_start(&mut self) -> Result<(), DocumentError> {
        if self.selection().is_some() {
            return self.backspace();
        }
        let start: CharIdx = self
            .line_start(self.cursor_line())
            .ok_or(DocumentError::Selection)?;
        if start == self.cursor {
            return self.backspace();
        }
        self.delete_range(start, self.cursor)
    }

    /// Cmd+Delete: the selection, else the line from the cursor to its end;
    /// at a line end, the line break after it.
    ///
    /// # Errors
    /// Returns a checked text error.
    pub fn delete_to_line_end(&mut self) -> Result<(), DocumentError> {
        if self.selection().is_some() {
            return self.delete();
        }
        let end: CharIdx = self.line_end(self.cursor_line())?;
        if end == self.cursor {
            return self.delete();
        }
        self.delete_range(self.cursor, end)
    }

    /// The char after a line's last visible char, before its line break.
    fn line_end(&self, index: usize) -> Result<CharIdx, DocumentError> {
        let start: CharIdx = self.line_start(index).ok_or(DocumentError::Selection)?;
        let text: String = self.line(index).ok_or(DocumentError::Selection)?;
        let visible: usize = text.trim_end_matches(['\r', '\n']).chars().count();
        Ok(CharIdx::new(start.raw().saturating_add(visible)))
    }

    /// Tab: with a selection, every line it touches moves in one level and
    /// the selection stays; without one, spaces to the next tab stop.
    ///
    /// # Errors
    /// Returns a checked text error.
    pub fn indent(&mut self) -> Result<(), DocumentError> {
        let Some((from, to)): Option<(CharIdx, CharIdx)> = self.selection() else {
            let start: CharIdx = self
                .line_start(self.cursor_line())
                .ok_or(DocumentError::Selection)?;
            let column: usize = self.cursor.raw().saturating_sub(start.raw());
            return self.insert(&" ".repeat(INDENT.len() - column % INDENT.len()));
        };
        let mut edits: Vec<CharEdit> = Vec::new();
        for line in self.lines_touched(from, to)? {
            let at: CharIdx = self.line_start(line).ok_or(DocumentError::Selection)?;
            edits.push(CharEdit {
                from: at,
                to: at,
                text: INDENT.to_owned(),
            });
        }
        self.reindent(&edits, (from, to))
    }

    /// Shift+Tab: every line the selection touches (or the cursor line)
    /// moves out one level: up to four leading spaces, or one tab.
    ///
    /// # Errors
    /// Returns a checked text error.
    pub fn outdent(&mut self) -> Result<(), DocumentError> {
        let (from, to): (CharIdx, CharIdx) = self.selection().unwrap_or((self.cursor, self.cursor));
        let mut edits: Vec<CharEdit> = Vec::new();
        for line in self.lines_touched(from, to)? {
            let at: CharIdx = self.line_start(line).ok_or(DocumentError::Selection)?;
            let taken: usize = leading_indent(&self.line(line).unwrap_or_default());
            if taken > 0 {
                edits.push(CharEdit {
                    from: at,
                    to: CharIdx::new(at.raw().saturating_add(taken)),
                    text: String::new(),
                });
            }
        }
        self.reindent(&edits, (from, to))
    }

    /// Cmd+A: the whole text.
    ///
    /// # Errors
    /// Returns a checked text error.
    pub fn select_all(&mut self) -> Result<(), DocumentError> {
        self.select(CharIdx::new(0), CharIdx::new(self.len_chars()))
    }

    /// Cmd+/: comment the lines the selection touches (or the cursor line)
    /// with `// ` after their indent, blank lines left alone; when every
    /// such line is commented already, uncomment them instead. One undo
    /// step; the selection stays.
    ///
    /// # Errors
    /// Returns a checked text error.
    pub fn toggle_line_comment(&mut self) -> Result<(), DocumentError> {
        let (from, to): (CharIdx, CharIdx) = self.selection().unwrap_or((self.cursor, self.cursor));
        let mut lines: Vec<(CharIdx, String)> = Vec::new();
        for line in self.lines_touched(from, to)? {
            let start: CharIdx = self.line_start(line).ok_or(DocumentError::Selection)?;
            lines.push((start, self.line(line).unwrap_or_default()));
        }
        let commented: bool = lines
            .iter()
            .filter(|(_, text): &&(CharIdx, String)| !text.trim().is_empty())
            .all(|(_, text): &(CharIdx, String)| text.trim_start().starts_with("//"));
        let mut edits: Vec<CharEdit> = Vec::new();
        for (start, text) in &lines {
            if text.trim().is_empty() {
                continue;
            }
            let indent: usize = text
                .chars()
                .take_while(|ch: &char| *ch == ' ' || *ch == '\t')
                .count();
            let at: CharIdx = CharIdx::new(start.raw().saturating_add(indent));
            let (to, text): (CharIdx, String) = if commented {
                let rest: &str = text.trim_start_matches([' ', '\t']);
                let taken: usize = if rest.starts_with(COMMENT) { 3 } else { 2 };
                (CharIdx::new(at.raw().saturating_add(taken)), String::new())
            } else {
                (at, COMMENT.to_owned())
            };
            edits.push(CharEdit { from: at, to, text });
        }
        self.reindent(&edits, (from, to))
    }

    /// The lines a selection `[from, to)` touches; a selection ending at a
    /// line's start leaves that line alone.
    fn lines_touched(&self, from: CharIdx, to: CharIdx) -> Result<Vec<usize>, DocumentError> {
        let first: usize = self
            .buffer
            .line_of_char(from)
            .ok_or(DocumentError::Selection)?;
        let mut last: usize = self
            .buffer
            .line_of_char(to)
            .ok_or(DocumentError::Selection)?;
        if last > first && self.line_start(last) == Some(to) {
            last = last.saturating_sub(1);
        }
        Ok((first..=last).collect())
    }

    /// Apply the line edits as one undo step; a selection is put back with
    /// each end moved by what was inserted or removed before it.
    fn reindent(
        &mut self,
        edits: &[CharEdit],
        (from, to): (CharIdx, CharIdx),
    ) -> Result<(), DocumentError> {
        if edits.is_empty() {
            return Ok(());
        }
        let selected: bool = self.selection().is_some();
        let forward: bool = self
            .anchor
            .is_none_or(|anchor: CharIdx| anchor <= self.cursor);
        self.replace_many(edits)?;
        if selected {
            let (start, end): (CharIdx, CharIdx) = (shifted(from, edits), shifted(to, edits));
            if forward {
                self.select(start, end)?;
            } else {
                self.select(end, start)?;
            }
        }
        Ok(())
    }
}

/// Up to four leading spaces, or one leading tab: what one outdent removes.
fn leading_indent(line: &str) -> usize {
    if line.starts_with('\t') {
        return 1;
    }
    line.chars()
        .take_while(|ch: &char| *ch == ' ')
        .take(INDENT.len())
        .count()
}

/// Where `at` lands once the sorted `edits` (none overlapping) are applied:
/// moved by every edit before it; a removal it sits in puts it at the
/// removal's start; an insert at its own place leaves it be.
fn shifted(at: CharIdx, edits: &[CharEdit]) -> CharIdx {
    let mut moved: usize = at.raw();
    for edit in edits {
        let removed: usize = edit.to.raw().saturating_sub(edit.from.raw());
        let added: usize = edit.text.chars().count();
        if (removed > 0 && edit.to <= at) || (removed == 0 && edit.from < at) {
            moved = moved.saturating_sub(removed).saturating_add(added);
        } else if edit.from < at && at < edit.to {
            moved = moved.saturating_sub(at.raw().saturating_sub(edit.from.raw()));
        }
    }
    CharIdx::new(moved)
}
