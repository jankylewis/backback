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

#[test]
fn select_all_takes_the_whole_text() {
    let mut d: Document = doc("ab\ncd\n", 1);
    d.select_all().unwrap();
    assert_eq!(d.selection(), Some((CharIdx::new(0), CharIdx::new(6))));
    d.insert("x").unwrap();
    assert_eq!(d.text(), "x");
}

#[test]
fn cmd_slash_comments_the_touched_lines_after_their_indent_and_back() {
    let text: &str = "class C\n{\n    int a;\n\n    int b;\n}\n";
    let mut d: Document = doc(text, 0);
    d.select(CharIdx::new(10), CharIdx::new(26)).unwrap();
    d.toggle_line_comment().unwrap();
    assert_eq!(
        d.text(),
        "class C\n{\n    // int a;\n\n    // int b;\n}\n",
        "after the indent; blank lines stay bare"
    );
    assert_eq!(d.selection(), Some((CharIdx::new(10), CharIdx::new(29))));
    d.toggle_line_comment().unwrap();
    assert_eq!(d.text(), text, "all commented: uncomment");
    assert_eq!(d.selection(), Some((CharIdx::new(10), CharIdx::new(26))));
    d.undo().unwrap();
    assert_eq!(
        d.text(),
        "class C\n{\n    // int a;\n\n    // int b;\n}\n",
        "one undo step each"
    );
    let mut one: Document = doc("    x;\n", 5);
    one.toggle_line_comment().unwrap();
    assert_eq!(one.text(), "    // x;\n", "no selection: the cursor line");
    assert_eq!(one.cursor().raw(), 8, "the cursor keeps its char");
}

#[test]
fn indent_lines_moves_the_cursor_line_in_at_its_start_and_outdent_back() {
    let mut d: Document = doc("a\n    int x;\nb\n", 10);
    d.indent_lines().unwrap();
    assert_eq!(
        d.text(),
        "a\n        int x;\nb\n",
        "in one level at the line start"
    );
    assert_eq!(d.cursor().raw(), 14, "the cursor stays on its text");
    d.outdent().unwrap();
    assert_eq!(d.text(), "a\n    int x;\nb\n", "and out again");
    d.undo().unwrap();
    assert_eq!(d.text(), "a\n        int x;\nb\n", "one undo step each");
}

#[test]
fn cmd_shift_k_deletes_the_touched_lines_with_their_break() {
    let mut d: Document = doc("a\nb\nc\n", 2);
    d.delete_lines().unwrap();
    assert_eq!(d.text(), "a\nc\n");
    assert_eq!(d.cursor().raw(), 2);
    let mut last: Document = doc("a\nb", 2);
    last.delete_lines().unwrap();
    assert_eq!(last.text(), "a", "the last line takes the break before it");
    let mut crlf: Document = doc("a\r\nb\r\nc\r\n", 3);
    crlf.delete_lines().unwrap();
    assert_eq!(crlf.text(), "a\r\nc\r\n", "CRLF goes as one break");
    let mut sel: Document = doc("a\nb\nc\nd\n", 0);
    sel.select(CharIdx::new(2), CharIdx::new(5)).unwrap();
    sel.delete_lines().unwrap();
    assert_eq!(sel.text(), "a\nd\n", "every touched line goes");
    sel.undo().unwrap();
    assert_eq!(sel.text(), "a\nb\nc\nd\n", "one undo step");
}

#[test]
fn cmd_enter_and_cmd_shift_enter_open_a_line_below_and_above_with_its_indent() {
    let mut below: Document = doc("{\n    int x;\n}\n", 6);
    below.insert_line_below().unwrap();
    assert_eq!(below.text(), "{\n    int x;\n    \n}\n");
    assert_eq!(
        below.cursor().raw(),
        17,
        "on the new line, after its indent"
    );
    below.undo().unwrap();
    assert_eq!(below.text(), "{\n    int x;\n}\n", "one undo step");
    let mut above: Document = doc("{\n    int x;\n}\n", 6);
    above.insert_line_above().unwrap();
    assert_eq!(above.text(), "{\n    \n    int x;\n}\n");
    assert_eq!(above.cursor().raw(), 6, "on the new line, after its indent");
    let mut crlf: Document = doc("a\r\nb\r\n", 0);
    crlf.insert_line_below().unwrap();
    assert_eq!(crlf.text(), "a\r\n\r\nb\r\n", "the file's own line break");
}
