//! One open document: its mirror text, its version, and how an edit becomes
//! an incremental `didChange` event.

use bb_text::{Buffer, CharIdx, LspPoint};
use lsp_types::TextDocumentContentChangeEvent;

use super::Span;
use crate::IntelError;

/// An LSP document version. Starts at 1 and only goes up.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocVersion(i32);

impl DocVersion {
    /// The version a document opens with.
    #[must_use]
    pub const fn first() -> Self {
        Self(1)
    }

    /// The version after this one; `None` past `i32::MAX`.
    #[must_use]
    pub fn next(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }

    /// A version read from the wire; `None` below 1, which no document has.
    #[must_use]
    pub fn from_wire(raw: i32) -> Option<Self> {
        (raw >= 1).then_some(Self(raw))
    }

    /// The raw number, for the wire.
    #[must_use]
    pub const fn raw(self) -> i32 {
        self.0
    }
}

/// Replace the chars in `[from, to)` with `text`. An insert has `from == to`;
/// a delete has empty `text`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edit {
    /// First char replaced.
    pub from: CharIdx,
    /// Char after the last one replaced.
    pub to: CharIdx,
    /// The new text.
    pub text: String,
}

/// The mirror of what the server holds for one document.
#[derive(Clone, Debug)]
pub(crate) struct OpenDoc {
    pub(crate) buffer: Buffer,
    pub(crate) version: DocVersion,
}

impl OpenDoc {
    pub(crate) fn new(text: &str) -> Self {
        Self {
            buffer: Buffer::from_text(text),
            version: DocVersion::first(),
        }
    }

    /// Apply `edits` in order. Either every edit applies and the events come
    /// back with the new version, or nothing changes.
    pub(crate) fn apply(
        &mut self,
        edits: &[Edit],
    ) -> Result<Option<(DocVersion, Vec<TextDocumentContentChangeEvent>)>, IntelError> {
        let mut next: Buffer = self.buffer.clone();
        let mut events: Vec<TextDocumentContentChangeEvent> = Vec::with_capacity(edits.len());
        for edit in edits {
            events.push(apply_one(&mut next, edit)?);
        }
        let Some(version): Option<DocVersion> = self.version.next() else {
            return Ok(None);
        };
        self.buffer = next;
        self.version = version;
        Ok(Some((version, events)))
    }
}

/// The event for `edit`, positioned in `buffer` before the edit, then the edit.
fn apply_one(
    buffer: &mut Buffer,
    edit: &Edit,
) -> Result<TextDocumentContentChangeEvent, IntelError> {
    let edit: &Edit = &widen_crlf(buffer, edit);
    let start: LspPoint = buffer.to_lsp(edit.from)?;
    let end: LspPoint = buffer.to_lsp(edit.to)?;
    buffer.remove(edit.from, edit.to)?;
    buffer.insert(edit.from, &edit.text)?;
    Ok(TextDocumentContentChangeEvent {
        range: Some(Span { start, end }.to_lsp()),
        range_length: None,
        text: edit.text.clone(),
    })
}

/// LSP cannot point between the CR and the LF of one line break: the server
/// clamps such a position to the line end, before the CR, and its text drifts
/// from the mirror. Move a boundary that falls there to cover the whole pair
/// and put the covered CR or LF back into the text, so the result is the same.
fn widen_crlf(buffer: &Buffer, edit: &Edit) -> Edit {
    let inside: bool = inside_crlf(buffer, edit.from);
    let from: CharIdx = if inside {
        CharIdx::new(edit.from.raw().saturating_sub(1))
    } else {
        edit.from
    };
    let to_inside: bool = inside_crlf(buffer, edit.to);
    let to: CharIdx = if to_inside {
        CharIdx::new(edit.to.raw().saturating_add(1))
    } else {
        edit.to
    };
    let mut text: String = String::with_capacity(edit.text.len().saturating_add(2));
    if inside {
        text.push('\r');
    }
    text.push_str(&edit.text);
    if to_inside {
        text.push('\n');
    }
    Edit { from, to, text }
}

/// Whether `at` sits between a CR and the LF right after it.
fn inside_crlf(buffer: &Buffer, at: CharIdx) -> bool {
    let before: Option<char> = at
        .raw()
        .checked_sub(1)
        .and_then(|i: usize| buffer.char_at(CharIdx::new(i)));
    before == Some('\r') && buffer.char_at(at) == Some('\n')
}

/// The event that replaces the whole document with `buffer`.
pub(crate) fn full_text(buffer: &Buffer) -> TextDocumentContentChangeEvent {
    TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: buffer.to_text(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edit(from: usize, to: usize, text: &str) -> Edit {
        Edit {
            from: CharIdx::new(from),
            to: CharIdx::new(to),
            text: text.to_owned(),
        }
    }

    #[test]
    fn edits_become_events_positioned_before_each_edit_in_utf16() {
        let mut doc: OpenDoc = OpenDoc::new("a😀b\ncd");
        let (version, events): (DocVersion, Vec<TextDocumentContentChangeEvent>) = doc
            .apply(&[edit(2, 3, "X"), edit(4, 5, "")])
            .unwrap()
            .unwrap();
        assert_eq!(version, DocVersion(2));
        assert_eq!(doc.buffer.to_text(), "a😀X\nd");
        let first: lsp_types::Range = events[0].range.unwrap();
        assert_eq!((first.start.character, first.end.character), (3, 4));
        let second: lsp_types::Range = events[1].range.unwrap();
        assert_eq!((second.start.line, second.start.character), (1, 0));
    }

    #[test]
    fn a_failing_edit_leaves_text_and_version_unchanged() {
        let mut doc: OpenDoc = OpenDoc::new("abc");
        let failed: Result<_, IntelError> = doc.apply(&[edit(0, 1, "z"), edit(9, 9, "x")]);
        assert!(failed.is_err());
        assert_eq!(doc.buffer.to_text(), "abc");
        assert_eq!(doc.version, DocVersion::first());
    }

    fn sent(text: &str, one: Edit) -> (String, (u32, u32, u32, u32), String) {
        let mut doc: OpenDoc = OpenDoc::new(text);
        let (_, events): (DocVersion, Vec<TextDocumentContentChangeEvent>) =
            doc.apply(&[one]).unwrap().unwrap();
        let r: lsp_types::Range = events[0].range.unwrap();
        let at: (u32, u32, u32, u32) =
            (r.start.line, r.start.character, r.end.line, r.end.character);
        (doc.buffer.to_text(), at, events[0].text.clone())
    }

    #[test]
    fn an_edit_boundary_inside_a_crlf_pair_is_widened_to_the_whole_pair() {
        let starts: (String, (u32, u32, u32, u32), String) = sent("ab\r\ncd", edit(3, 4, "X"));
        assert_eq!(
            starts,
            ("ab\rXcd".to_owned(), (0, 2, 1, 0), "\rX".to_owned())
        );
        let inserts: (String, (u32, u32, u32, u32), String) = sent("ab\r\ncd", edit(3, 3, "Y"));
        assert_eq!(
            inserts,
            ("ab\rY\ncd".to_owned(), (0, 2, 1, 0), "\rY\n".to_owned())
        );
        let ends: (String, (u32, u32, u32, u32), String) = sent("ab\r\ncd", edit(1, 3, "Z"));
        assert_eq!(ends, ("aZ\ncd".to_owned(), (0, 1, 1, 0), "Z\n".to_owned()));
    }

    #[test]
    fn the_version_stops_at_the_top_instead_of_wrapping() {
        let mut doc: OpenDoc = OpenDoc::new("abc");
        doc.version = DocVersion(i32::MAX);
        assert!(doc.apply(&[edit(0, 0, "x")]).unwrap().is_none());
        assert_eq!(doc.buffer.to_text(), "abc");
    }
}
