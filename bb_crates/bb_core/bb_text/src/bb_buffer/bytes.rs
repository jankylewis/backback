//! Byte-level views of the buffer: chunked writes, byte offsets and the LF
//! rows tree-sitter uses.

use ropey::Rope;

use super::Buffer;
use crate::position::CharIdx;

impl Buffer {
    /// Write the whole text without one large allocation.
    ///
    /// # Errors
    /// Returns the writer's error.
    pub fn write_to(&self, out: impl std::io::Write) -> std::io::Result<()> {
        self.rope.write_to(out)
    }

    /// The rope itself, for chunked readers.
    #[must_use]
    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    /// Start byte offset of a line.
    #[must_use]
    pub fn line_start_byte(&self, line: usize) -> Option<usize> {
        self.rope.try_line_to_byte(line).ok()
    }

    /// Byte offset, LF row and byte column of a char index, as tree-sitter
    /// counts them (only `\n` starts a row). O(log n) when `lf_only` says
    /// the text has no other line break; a full scan otherwise.
    #[must_use]
    pub fn byte_point(&self, at: CharIdx, lf_only: bool) -> Option<(usize, usize, usize)> {
        let byte: usize = self.rope.try_char_to_byte(at.raw()).ok()?;
        if lf_only {
            let row: usize = self.rope.try_byte_to_line(byte).ok()?;
            let line_byte: usize = self.rope.try_line_to_byte(row).ok()?;
            return Some((byte, row, byte.checked_sub(line_byte)?));
        }
        let mut row: usize = 0;
        let mut line_byte: usize = 0;
        for (index, value) in self.rope.get_byte_slice(..byte)?.bytes().enumerate() {
            if value == b'\n' {
                row = row.saturating_add(1);
                line_byte = index.saturating_add(1);
            }
        }
        Some((byte, row, byte.checked_sub(line_byte)?))
    }

    /// Whether `text` holds a line break besides LF and CRLF. One linear pass.
    #[must_use]
    pub fn text_has_other_break(text: &str) -> bool {
        let mut chars: std::iter::Peekable<std::str::Chars<'_>> = text.chars().peekable();
        while let Some(ch) = chars.next() {
            let other: bool = match ch {
                '\r' => chars.peek() != Some(&'\n'),
                '\u{0b}' | '\u{0c}' | '\u{85}' | '\u{2028}' | '\u{2029}' => true,
                _ => false,
            };
            if other {
                return true;
            }
        }
        false
    }

    /// Whether chars in `[from - 1, to + 1)` hold a line break the rope counts
    /// but tree-sitter does not (a lone CR, VT, FF, NEL, LS or PS).
    #[must_use]
    pub fn other_break_near(&self, from: CharIdx, to: CharIdx) -> bool {
        let first: usize = from.raw().saturating_sub(1);
        let last: usize = to.raw().saturating_add(1).min(self.rope.len_chars());
        (first..last).any(|index: usize| match self.rope.get_char(index) {
            Some('\r') => self.rope.get_char(index.saturating_add(1)) != Some('\n'),
            Some(ch) => matches!(ch, '\u{0b}' | '\u{0c}' | '\u{85}' | '\u{2028}' | '\u{2029}'),
            None => false,
        })
    }
}
