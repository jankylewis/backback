//! BB-BUG-007: Home, End and the Cmd+arrow edges move the cursor.

use crate::document::Document;
use crate::CharIdx;

const TEXT: &str = "class A\n    int x = 1;\n\n}\n";

fn at(text: &str, cursor: usize) -> Document {
    let mut doc: Document = Document::from_text(text, None);
    doc.set_cursor(CharIdx::new(cursor)).unwrap();
    doc
}

#[test]
fn end_goes_to_the_visible_line_end_and_home_to_its_first_char() {
    let mut doc: Document = at(TEXT, 14);
    doc.end(false).unwrap();
    assert_eq!(doc.cursor().raw(), 22, "after `;`, before the newline");
    doc.home(false).unwrap();
    assert_eq!(doc.cursor().raw(), 12, "first non-blank: `int`");
    doc.home(false).unwrap();
    assert_eq!(doc.cursor().raw(), 8, "Home again: column 0");
    doc.home(false).unwrap();
    assert_eq!(doc.cursor().raw(), 12, "and back to the first non-blank");
}

#[test]
fn end_on_an_empty_line_and_on_the_last_line_stays_on_that_line() {
    let mut doc: Document = at(TEXT, 23);
    doc.end(false).unwrap();
    assert_eq!(doc.cursor().raw(), 23, "empty line: end is its start");
    let mut last: Document = at("a\nbc", 2);
    last.end(false).unwrap();
    assert_eq!(last.cursor().raw(), 4, "no trailing newline");
}

#[test]
fn shift_end_selects_to_the_line_end() {
    let mut doc: Document = at(TEXT, 12);
    doc.end(true).unwrap();
    assert_eq!(doc.selection(), Some((CharIdx::new(12), CharIdx::new(22))));
}

#[test]
fn document_edges_go_to_the_first_and_last_char() {
    let mut doc: Document = at(TEXT, 14);
    doc.document_edge(true, false).unwrap();
    assert_eq!(doc.cursor().raw(), TEXT.chars().count());
    doc.document_edge(false, true).unwrap();
    assert_eq!(doc.cursor().raw(), 0);
    assert!(doc.selection().is_some(), "shift keeps a selection");
}
