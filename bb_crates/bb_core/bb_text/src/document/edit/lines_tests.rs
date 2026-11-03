//! BB-BUG-014 and BB-BUG-015: the line-wise edits.

use crate::{CharIdx, Document};

fn doc(text: &str, cursor: usize) -> Document {
    let mut doc: Document = Document::from_text(text, None);
    doc.set_cursor(CharIdx::new(cursor)).unwrap();
    doc
}

#[test]
fn cmd_backspace_and_cmd_delete_remove_to_the_line_edges() {
    let mut d: Document = doc("ab\n    int a = 1;\ncd\n", 11);
    d.delete_to_line_start().unwrap();
    assert_eq!(d.text(), "ab\na = 1;\ncd\n", "to the line start");
    assert_eq!(d.cursor().raw(), 3);
    d.delete_to_line_end().unwrap();
    assert_eq!(d.text(), "ab\n\ncd\n", "to the line end");
    d.delete_to_line_end().unwrap();
    assert_eq!(d.text(), "ab\ncd\n", "at the line end: the break goes");
    d.delete_to_line_start().unwrap();
    assert_eq!(d.text(), "abcd\n", "at a line start: joins the line above");
    let mut s: Document = doc("abc def\n", 0);
    s.select(CharIdx::new(1), CharIdx::new(3)).unwrap();
    s.delete_to_line_start().unwrap();
    assert_eq!(s.text(), "a def\n", "a selection goes, nothing more");
}

#[test]
fn tab_indents_the_selected_lines_and_keeps_the_selection_as_one_undo_step() {
    let text: &str = "class C\n{\n    int a;\n    int b;\n}\n";
    let (l2, l3): (usize, usize) = (10, 21);
    let mut d: Document = doc(text, 0);
    d.select(CharIdx::new(l2 + 2), CharIdx::new(l3)).unwrap();
    d.indent().unwrap();
    assert_eq!(
        d.text(),
        "class C\n{\n        int a;\n    int b;\n}\n",
        "a selection ending at a line start leaves that line alone"
    );
    assert_eq!(
        d.selection(),
        Some((CharIdx::new(l2 + 6), CharIdx::new(l3 + 4))),
        "the selection follows its chars"
    );
    d.undo().unwrap();
    assert_eq!(d.text(), text, "one undo step");
    d.select(CharIdx::new(l2), CharIdx::new(l3 + 5)).unwrap();
    d.indent().unwrap();
    assert_eq!(d.text(), "class C\n{\n        int a;\n        int b;\n}\n");
    assert_eq!(
        d.selection(),
        Some((CharIdx::new(l2), CharIdx::new(l3 + 13))),
        "a start at column 0 stays there"
    );
}

#[test]
fn shift_tab_outdents_spaces_or_a_tab_and_leaves_bare_lines() {
    let text: &str = "a\n        b\n  c\n\td\ne\n";
    let mut d: Document = doc(text, 0);
    d.select(CharIdx::new(2), CharIdx::new(20)).unwrap();
    d.outdent().unwrap();
    assert_eq!(d.text(), "a\n    b\nc\nd\ne\n");
    assert_eq!(
        d.selection(),
        Some((CharIdx::new(2), CharIdx::new(13))),
        "the selection shrinks with the removed indents"
    );
    d.undo().unwrap();
    assert_eq!(d.text(), text, "one undo step");
    let mut one: Document = doc("    int a;\n", 8);
    one.outdent().unwrap();
    assert_eq!(one.text(), "int a;\n", "no selection: the cursor line");
    assert_eq!(one.cursor().raw(), 4, "the cursor keeps its char");
}

#[test]
fn tab_without_a_selection_types_spaces_to_the_next_stop() {
    let mut d: Document = doc("ab\n", 1);
    d.indent().unwrap();
    assert_eq!(d.text(), "a   b\n", "three spaces reach column 4");
    let mut at_stop: Document = doc("abcd\n", 4);
    at_stop.indent().unwrap();
    assert_eq!(at_stop.text(), "abcd    \n", "a full stop at a stop");
}
