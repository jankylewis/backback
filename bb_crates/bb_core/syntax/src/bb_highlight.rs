//! Visible-range C# captures and incremental tree updates, read from the
//! editor's rope in chunks: no keystroke copies or rescans the whole text.

use bb_text::{BytePoint, Rope, TextChange};
use tree_sitter::{
    InputEdit, Node, Point, QueryCapture, QueryCursor, StreamingIterator, TextProvider, Tree,
};

use crate::{CSharpSyntax, SyntaxError};

/// Colour role for a syntax capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightKind {
    /// C# keyword.
    Keyword,
    /// Text literal.
    String,
    /// Comment.
    Comment,
    /// Named type.
    Type,
    /// Other capture.
    Plain,
}

/// Byte range and colour role of one capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighlightSpan {
    /// Inclusive start byte.
    pub start: usize,
    /// Exclusive end byte.
    pub end: usize,
    /// Theme role.
    pub kind: HighlightKind,
}

impl CSharpSyntax {
    /// Parse the rope. Pass the edited previous tree for an incremental parse.
    ///
    /// # Errors
    /// [`SyntaxError::NoTree`] when the parser returns nothing.
    pub fn parse_rope(&mut self, rope: &Rope, old: Option<&Tree>) -> Result<Tree, SyntaxError> {
        self.parser
            .parse_with_options(
                &mut |byte: usize, _: Point| -> &[u8] { chunk_from(rope, byte) },
                old,
                None,
            )
            .ok_or(SyntaxError::NoTree)
    }

    /// Parse only `window` (start and end of a byte range) of the rope, or the
    /// whole rope when `None`. Node offsets stay those of the whole text.
    ///
    /// # Errors
    /// [`SyntaxError::Window`] for a bad window, [`SyntaxError::NoTree`] when
    /// the parser returns nothing.
    pub fn parse_rope_in(
        &mut self,
        rope: &Rope,
        old: Option<&Tree>,
        window: Option<(BytePoint, BytePoint)>,
    ) -> Result<Tree, SyntaxError> {
        let ranges: Vec<tree_sitter::Range> = window
            .map(|(start, end): (BytePoint, BytePoint)| tree_sitter::Range {
                start_byte: start.byte,
                end_byte: end.byte,
                start_point: point(start),
                end_point: point(end),
            })
            .into_iter()
            .collect();
        self.parser.set_included_ranges(&ranges)?;
        self.parse_rope(rope, old)
    }

    /// Tell `tree` about one text edit, so the next parse reuses it. O(depth).
    pub fn edit(tree: &mut Tree, change: &TextChange) {
        tree.edit(&InputEdit {
            start_byte: change.start.byte,
            old_end_byte: change.old_end.byte,
            new_end_byte: change.new_end.byte,
            start_position: point(change.start),
            old_end_position: point(change.old_end),
            new_end_position: point(change.new_end),
        });
    }

    /// Captures that overlap the visible byte range.
    #[must_use]
    pub fn highlight_spans(
        &self,
        tree: &Tree,
        rope: &Rope,
        visible: std::ops::Range<usize>,
    ) -> Vec<HighlightSpan> {
        let mut cursor: QueryCursor = QueryCursor::new();
        cursor.set_byte_range(visible.clone());
        let mut captures: tree_sitter::QueryCaptures<'_, '_, RopeText<'_>, &[u8]> =
            cursor.captures(&self.query, tree.root_node(), RopeText(rope));
        let mut spans: Vec<HighlightSpan> = Vec::new();
        while let Some((found, capture_index)) = captures.next() {
            let Some(capture): Option<&QueryCapture<'_>> = found.captures.get(*capture_index)
            else {
                continue;
            };
            let name: &str = self
                .query
                .capture_names()
                .get(usize::try_from(capture.index).unwrap_or(usize::MAX))
                .copied()
                .unwrap_or("");
            let start: usize = capture.node.start_byte().max(visible.start);
            let end: usize = capture.node.end_byte().min(visible.end);
            if start < end {
                spans.push(HighlightSpan {
                    start,
                    end,
                    kind: classify(name),
                });
            }
        }
        spans
    }
}

/// Node text for query predicates, as rope chunks.
struct RopeText<'a>(&'a Rope);

impl<'a> TextProvider<&'a [u8]> for RopeText<'a> {
    type I = std::iter::Flatten<
        std::option::IntoIter<std::iter::Map<ropey::iter::Chunks<'a>, fn(&'a str) -> &'a [u8]>>,
    >;

    fn text(&mut self, node: Node<'_>) -> Self::I {
        let bytes: fn(&'a str) -> &'a [u8] = str::as_bytes;
        self.0
            .get_byte_slice(node.byte_range())
            .map(|slice: ropey::RopeSlice<'a>| slice.chunks().map(bytes))
            .into_iter()
            .flatten()
    }
}

/// The rope's bytes from `byte` to the end of its chunk; empty past the end.
fn chunk_from(rope: &Rope, byte: usize) -> &[u8] {
    rope.get_chunk_at_byte(byte)
        .and_then(|(chunk, start, _, _): (&str, usize, usize, usize)| {
            chunk.as_bytes().get(byte.checked_sub(start)?..)
        })
        .unwrap_or(&[])
}

fn point(at: BytePoint) -> Point {
    Point::new(at.row, at.column)
}

fn classify(name: &str) -> HighlightKind {
    if name.starts_with("keyword") {
        HighlightKind::Keyword
    } else if name.starts_with("string") {
        HighlightKind::String
    } else if name.starts_with("comment") {
        HighlightKind::Comment
    } else if name.starts_with("type") || name == "constructor" {
        HighlightKind::Type
    } else {
        HighlightKind::Plain
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bb_text::{CharIdx, Document};
    use proptest::prelude::*;

    fn reparsed(syntax: &mut CSharpSyntax, doc: &mut Document, tree: &Tree) -> Tree {
        let mut edited: Tree = tree.clone();
        for change in doc.take_changes() {
            CSharpSyntax::edit(&mut edited, &change);
        }
        syntax.parse_rope(doc.rope(), Some(&edited)).unwrap()
    }

    #[test]
    fn edited_tree_and_visible_captures_follow_new_text() {
        let mut syntax: CSharpSyntax = CSharpSyntax::new().unwrap();
        let mut doc: Document = Document::from_text("class A { string S = \"old\"; }", None);
        let first: Tree = syntax.parse_rope(doc.rope(), None).unwrap();
        doc.select(CharIdx::new(22), CharIdx::new(25)).unwrap();
        doc.insert("new").unwrap();
        let next: Tree = reparsed(&mut syntax, &mut doc, &first);
        assert!(!next.root_node().has_error());
        let text: String = doc.text();
        let spans: Vec<HighlightSpan> = syntax.highlight_spans(&next, doc.rope(), 0..text.len());
        assert!(spans
            .iter()
            .any(|span: &HighlightSpan| span.kind == HighlightKind::String
                && text.get(span.start..span.end).unwrap_or("").contains("new")));
        assert!(syntax
            .highlight_spans(&next, doc.rope(), 0..5)
            .iter()
            .all(|span: &HighlightSpan| span.end <= 5));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]
        #[test]
        fn incremental_parse_equals_a_fresh_parse(
            edits in proptest::collection::vec((0usize..400, 0u8..4), 1..25),
        ) {
            let source: &str = "namespace N;\r\nclass A\r\n{\r\n    int F() => 1; // c\r\n    string S = \"x\";\r\n}\r\n";
            let mut syntax: CSharpSyntax = CSharpSyntax::new().unwrap();
            let mut doc: Document = Document::from_text(source, None);
            let mut tree: Tree = syntax.parse_rope(doc.rope(), None).unwrap();
            for (at, kind) in edits {
                doc.set_cursor(CharIdx::new(at % (doc.len_chars() + 1))).unwrap();
                match kind {
                    0 => doc.insert("{").unwrap(),
                    1 => doc.backspace().unwrap(),
                    2 => doc.enter().unwrap(),
                    _ => doc.undo().unwrap(),
                }
                tree = reparsed(&mut syntax, &mut doc, &tree);
                let fresh: Tree = syntax.parse(&doc.text(), None).unwrap();
                prop_assert_eq!(tree.root_node().to_sexp(), fresh.root_node().to_sexp());
                prop_assert_eq!(tree.root_node().end_position(), fresh.root_node().end_position());
            }
        }
    }
}
