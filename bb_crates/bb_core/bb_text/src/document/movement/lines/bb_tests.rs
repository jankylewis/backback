use super::super::super::super::Document;
use crate::CharIdx;

fn doc(text: &str, cursor: usize) -> Document {
    let mut doc: Document = Document::from_text(text, None);
    doc.set_cursor(CharIdx::new(cursor)).unwrap();
    doc
}

#[test]
fn alt_down_and_up_swap_the_line_and_keep_the_cursor_on_it() {
    let mut d: Document = doc("a\nbb\nc\n", 3);
    d.move_lines(false).unwrap();
    assert_eq!(d.text(), "a\nc\nbb\n");
    assert_eq!(d.cursor(), CharIdx::new(5), "still after the first b");
    d.move_lines(true).unwrap();
    d.move_lines(true).unwrap();
    assert_eq!(d.text(), "bb\na\nc\n");
    assert_eq!(d.cursor(), CharIdx::new(1));
    d.move_lines(true).unwrap();
    assert_eq!(d.text(), "bb\na\nc\n", "the first line does not move up");
    d.undo().unwrap();
    assert_eq!(d.text(), "a\nbb\nc\n", "one undo step per key");
}

#[test]
fn a_last_line_without_a_break_moves_and_takes_one_only_where_needed() {
    let mut d: Document = doc("a\nb", 2);
    d.move_lines(true).unwrap();
    assert_eq!(d.text(), "b\na");
    assert_eq!(d.cursor(), CharIdx::new(0));
    d.move_lines(false).unwrap();
    assert_eq!(d.text(), "a\nb");
    d.move_lines(false).unwrap();
    assert_eq!(d.text(), "a\nb", "the last line does not move down");
}

#[test]
fn crlf_lines_keep_their_breaks() {
    let mut d: Document = doc("a\r\nb\r\nc", 0);
    d.move_lines(false).unwrap();
    assert_eq!(d.text(), "b\r\na\r\nc");
    let mut e: Document = doc("a\r\nb", 3);
    e.copy_lines(false).unwrap();
    assert_eq!(e.text(), "a\r\nb\r\nb");
}

#[test]
fn a_selection_over_lines_moves_whole_and_stays_selected() {
    let mut d: Document = doc("1\n2\n3\n4\n", 0);
    d.select(CharIdx::new(2), CharIdx::new(5)).unwrap();
    d.move_lines(false).unwrap();
    assert_eq!(d.text(), "1\n4\n2\n3\n");
    assert_eq!(d.selection(), Some((CharIdx::new(4), CharIdx::new(7))));
}

#[test]
fn shift_alt_down_and_up_copy_the_line() {
    let mut d: Document = doc("a\nb\n", 2);
    d.copy_lines(false).unwrap();
    assert_eq!(d.text(), "a\nb\nb\n");
    assert_eq!(d.cursor(), CharIdx::new(4), "on the copy below");
    let mut e: Document = doc("a\nb\n", 2);
    e.copy_lines(true).unwrap();
    assert_eq!(e.text(), "a\nb\nb\n");
    assert_eq!(e.cursor(), CharIdx::new(2), "on the copy above");
    let mut f: Document = doc("a\nb", 2);
    f.copy_lines(true).unwrap();
    assert_eq!(f.text(), "a\nb\nb");
    assert_eq!(f.cursor(), CharIdx::new(2));
    let mut g: Document = doc("a\nb", 2);
    g.copy_lines(false).unwrap();
    assert_eq!(g.text(), "a\nb\nb");
    assert_eq!(g.cursor(), CharIdx::new(4));
}
