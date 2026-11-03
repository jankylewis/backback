//! Alt+Up/Down move the lines the cursor or selection touches; Shift+Alt+
//! Up/Down copy them (BB-BUG-021). Each is one undo step; a line keeps its
//! own break (CRLF too); a last line without a break gets one only where it
//! now has a line below it.

use super::super::super::{Document, DocumentError};
use crate::{CharEdit, CharIdx};

impl Document {
    /// Alt+Up / Alt+Down: swap the touched lines with the line above or below.
    ///
    /// # Errors
    /// Returns a checked text error on an invalid range.
    pub fn move_lines(&mut self, up: bool) -> Result<(), DocumentError> {
        let (first, last): (usize, usize) = self.touched_lines()?;
        let (start, end, text, delta): (CharIdx, CharIdx, String, isize) = if up {
            let Some(above): Option<usize> = first.checked_sub(1) else {
                return Ok(());
            };
            let other: String = self.lines_text(above, above)?;
            let block: String = self.lines_text(first, last)?;
            let moved: String = joined(&block, &other);
            let shift: isize = signed(other.chars().count());
            (self.start_of(above)?, self.end_of(last)?, moved, -shift)
        } else {
            let below: usize = last.saturating_add(1);
            if below >= self.real_lines() {
                return Ok(());
            }
            let block: String = self.lines_text(first, last)?;
            let other: String = self.lines_text(below, below)?;
            let moved: String = joined(&other, &block);
            // The block now starts where `other` (and the break it gained) ends.
            let placed: usize = if ends_with_break(&other) {
                block.chars().count()
            } else {
                bare(&block).chars().count()
            };
            let shift: isize = signed(moved.chars().count().saturating_sub(placed));
            (self.start_of(first)?, self.end_of(below)?, moved, shift)
        };
        self.replace_keeping_selection(
            &CharEdit {
                from: start,
                to: end,
                text,
            },
            delta,
        )
    }

    /// Shift+Alt+Up / Shift+Alt+Down: copy the touched lines above or below;
    /// the cursor goes to the copy below, or stays on the copy above.
    ///
    /// # Errors
    /// Returns a checked text error on an invalid range.
    pub fn copy_lines(&mut self, up: bool) -> Result<(), DocumentError> {
        let (first, last): (usize, usize) = self.touched_lines()?;
        let block: String = self.lines_text(first, last)?;
        let start: CharIdx = self.start_of(first)?;
        let eol: &str = break_of(&block, self);
        // A last line without a break: above, the copy takes the break;
        // below, the original takes it.
        let edit: CharEdit = match (ends_with_break(&block), up) {
            (true, _) => CharEdit {
                from: start,
                to: start,
                text: block,
            },
            (false, true) => CharEdit {
                from: start,
                to: start,
                text: format!("{block}{eol}"),
            },
            (false, false) => {
                let at: CharIdx = self.end_of(last)?;
                CharEdit {
                    from: at,
                    to: at,
                    text: format!("{eol}{block}"),
                }
            }
        };
        let delta: isize = if up {
            0
        } else {
            signed(edit.text.chars().count())
        };
        self.replace_keeping_selection(&edit, delta)
    }

    /// The first and last line the cursor or selection touches.
    fn touched_lines(&self) -> Result<(usize, usize), DocumentError> {
        let (from, to): (CharIdx, CharIdx) = self.selection().unwrap_or((self.cursor, self.cursor));
        let lines: Vec<usize> = self.lines_touched(from, to)?;
        match (lines.first(), lines.last()) {
            (Some(&first), Some(&last)) => Ok((first, last)),
            _ => Err(DocumentError::Selection),
        }
    }

    /// Lines that hold text: a final empty line after the last break is not one.
    fn real_lines(&self) -> usize {
        let all: usize = self.len_lines();
        let trailing: bool = self
            .line(all.saturating_sub(1))
            .is_some_and(|line: String| line.is_empty());
        if trailing && all > 1 {
            all.saturating_sub(1)
        } else {
            all
        }
    }

    fn start_of(&self, line: usize) -> Result<CharIdx, DocumentError> {
        self.line_start(line).ok_or(DocumentError::Selection)
    }

    /// The char after `line`'s break, or the text's end.
    fn end_of(&self, line: usize) -> Result<CharIdx, DocumentError> {
        let next: usize = line.saturating_add(1);
        if next < self.len_lines() {
            self.start_of(next)
        } else {
            Ok(CharIdx::new(self.len_chars()))
        }
    }

    /// Lines `first..=last` with their breaks.
    fn lines_text(&self, first: usize, last: usize) -> Result<String, DocumentError> {
        let (from, to): (CharIdx, CharIdx) = (self.start_of(first)?, self.end_of(last)?);
        Ok(self.rope().slice(from.raw()..to.raw()).to_string())
    }

    /// One undo step for `edit`; the cursor and anchor move by `delta`.
    fn replace_keeping_selection(
        &mut self,
        edit: &CharEdit,
        delta: isize,
    ) -> Result<(), DocumentError> {
        let (cursor, anchor): (CharIdx, Option<CharIdx>) = (self.cursor, self.anchor);
        self.replace_many(std::slice::from_ref(edit))?;
        match anchor {
            Some(anchor) => self.select(shifted(anchor, delta), shifted(cursor, delta)),
            None => self.set_cursor(shifted(cursor, delta)),
        }
    }
}

/// `first` above `second`: `first` gets a break if it was the last line.
fn joined(first: &str, second: &str) -> String {
    if ends_with_break(first) {
        return format!("{first}{second}");
    }
    let body: &str = bare(second);
    let eol: &str = second.get(body.len()..).unwrap_or_default();
    format!("{first}{eol}{body}")
}

/// `text` without its final break.
fn bare(text: &str) -> &str {
    text.strip_suffix("\r\n")
        .or_else(|| text.strip_suffix('\n'))
        .unwrap_or(text)
}

fn ends_with_break(text: &str) -> bool {
    text.ends_with('\n')
}

/// The break a new line copied from `block` takes: the block's own, else
/// the line above's, else LF.
fn break_of(block: &str, doc: &Document) -> &'static str {
    let crlf: bool = block.contains("\r\n")
        || doc
            .line(doc.cursor_line().saturating_sub(1))
            .is_some_and(|line: String| line.ends_with("\r\n"));
    if crlf {
        "\r\n"
    } else {
        "\n"
    }
}

fn shifted(at: CharIdx, delta: isize) -> CharIdx {
    CharIdx::new(at.raw().saturating_add_signed(delta))
}

fn signed(count: usize) -> isize {
    isize::try_from(count).unwrap_or(isize::MAX)
}

#[cfg(test)]
mod bb_tests;
