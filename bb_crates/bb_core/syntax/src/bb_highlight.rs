//! Visible-range C# captures and incremental tree updates.

use tree_sitter::{InputEdit, Point, QueryCapture, QueryCursor, StreamingIterator, Tree};

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

/// Edited byte interval in old and new text.
#[derive(Debug, Clone, Copy)]
pub struct ByteEdit {
    /// First changed byte.
    pub start: usize,
    /// Old exclusive end.
    pub old_end: usize,
    /// New exclusive end.
    pub new_end: usize,
}

impl CSharpSyntax {
    /// Reparse after one or more text edits, using an edited old tree.
    ///
    /// # Errors
    /// Returns [`SyntaxError::NoTree`] if parsing fails.
    pub fn reparse(&mut self, old: &Tree, before: &str, after: &str) -> Result<Tree, SyntaxError> {
        let start: usize = common_prefix(before, after);
        let suffix: usize = common_suffix(before, after, start);
        let old_end: usize = before.len().saturating_sub(suffix);
        let new_end: usize = after.len().saturating_sub(suffix);
        self.reparse_bytes(
            old,
            before,
            after,
            ByteEdit {
                start,
                old_end,
                new_end,
            },
        )
    }

    /// Reparse using byte offsets supplied by the text edit.
    ///
    /// # Errors
    /// Returns [`SyntaxError::NoTree`] if parsing fails.
    pub fn reparse_bytes(
        &mut self,
        old: &Tree,
        before: &str,
        after: &str,
        edit: ByteEdit,
    ) -> Result<Tree, SyntaxError> {
        let mut tree: Tree = old.clone();
        tree.edit(&InputEdit {
            start_byte: edit.start,
            old_end_byte: edit.old_end,
            new_end_byte: edit.new_end,
            start_position: point(before, edit.start),
            old_end_position: point(before, edit.old_end),
            new_end_position: point(after, edit.new_end),
        });
        self.parse(after, Some(&tree))
    }

    /// Captures that overlap the visible byte range.
    #[must_use]
    pub fn highlight_spans(
        &self,
        tree: &Tree,
        text: &str,
        visible: std::ops::Range<usize>,
    ) -> Vec<HighlightSpan> {
        let mut cursor: QueryCursor = QueryCursor::new();
        cursor.set_byte_range(visible.clone());
        let mut captures: tree_sitter::QueryCaptures<'_, '_, &[u8], &[u8]> =
            cursor.captures(&self.query, tree.root_node(), text.as_bytes());
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

fn common_prefix(before: &str, after: &str) -> usize {
    before
        .chars()
        .zip(after.chars())
        .take_while(|(a, b): &(char, char)| a == b)
        .map(|(ch, _): (char, char)| ch.len_utf8())
        .sum()
}

fn common_suffix(before: &str, after: &str, prefix: usize) -> usize {
    let limit: usize = before.len().min(after.len()).saturating_sub(prefix);
    before
        .chars()
        .rev()
        .zip(after.chars().rev())
        .take_while(|(a, b): &(char, char)| a == b)
        .map(|(ch, _): (char, char)| ch.len_utf8())
        .scan(0, |sum: &mut usize, len: usize| {
            *sum = sum.saturating_add(len);
            Some(*sum)
        })
        .take_while(|sum: &usize| *sum <= limit)
        .last()
        .unwrap_or(0)
}

fn point(text: &str, byte: usize) -> Point {
    let prefix: &str = text.get(..byte).unwrap_or("");
    let row: usize = prefix.bytes().filter(|value: &u8| *value == b'\n').count();
    let column: usize = prefix.rfind('\n').map_or(prefix.len(), |last: usize| {
        prefix.len().saturating_sub(last.saturating_add(1))
    });
    Point::new(row, column)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edited_tree_and_visible_captures_follow_new_text() {
        let mut syntax: CSharpSyntax = CSharpSyntax::new().unwrap();
        let old: &str = "class A { string S = \"old\"; }";
        let new: &str = "class A { string S = \"new\"; }";
        let first: Tree = syntax.parse(old, None).unwrap();
        let next: Tree = syntax.reparse(&first, old, new).unwrap();
        assert!(!next.root_node().has_error());
        let spans: Vec<HighlightSpan> = syntax.highlight_spans(&next, new, 0..new.len());
        assert!(spans
            .iter()
            .any(|span: &HighlightSpan| span.kind == HighlightKind::String
                && new.get(span.start..span.end).unwrap_or("").contains("new")));
        assert!(syntax
            .highlight_spans(&next, new, 0..5)
            .iter()
            .all(|span: &HighlightSpan| span.end <= 5));
    }
}
