//! BB-BUG-011: Option+Left/Right move by word, Option+Backspace/Delete
//! remove a word (VS Code / `IntelliJ` stops; a line break is its own stop).

use crate::document::Document;
use crate::CharIdx;

// l0 e1 t2 _3 t4..l8 _9 =10 _11 a12 .13 b14 (15 x16 )17 ;18 \r19 \n20 _21 _22 n23..t26
const TEXT: &str = "let total = a.b(x);\r\n  next";

fn at(cursor: usize) -> Document {
    let mut doc: Document = Document::from_text(TEXT, None);
    doc.set_cursor(CharIdx::new(cursor)).unwrap();
    doc
}

fn right_from(cursor: usize) -> usize {
    let mut doc: Document = at(cursor);
    doc.word_right(false).unwrap();
    doc.cursor().raw()
}

fn left_from(cursor: usize) -> usize {
    let mut doc: Document = at(cursor);
    doc.word_left(false).unwrap();
    doc.cursor().raw()
}

#[test]
fn option_right_stops_at_each_word_and_punctuation_end() {
    let stops: Vec<(usize, usize)> = vec![(0, 3), (3, 9), (9, 11), (11, 13), (13, 14), (17, 19)];
    for (from, to) in stops {
        assert_eq!(right_from(from), to, "word right from {from}");
    }
}

#[test]
fn option_right_steps_over_a_line_break_alone_then_skips_the_indent() {
    assert_eq!(right_from(19), 21, "over the whole \\r\\n, no further");
    assert_eq!(right_from(21), 27, "indent + `next`");
    assert_eq!(right_from(27), 27, "the document end stays");
}

#[test]
fn option_left_mirrors_it() {
    let stops: Vec<(usize, usize)> = vec![(27, 23), (23, 21), (21, 19), (19, 17), (4, 0), (0, 0)];
    for (from, to) in stops {
        assert_eq!(left_from(from), to, "word left from {from}");
    }
}

#[test]
fn shift_option_right_selects_the_word() {
    let mut doc: Document = at(0);
    doc.word_right(true).unwrap();
    assert_eq!(doc.selection(), Some((CharIdx::new(0), CharIdx::new(3))));
}

#[test]
fn option_backspace_and_delete_remove_one_word_stop() {
    let mut doc: Document = at(9);
    doc.delete_word_back().unwrap();
    assert_eq!(doc.text(), "let  = a.b(x);\r\n  next", "`total` only");
    let mut doc: Document = at(23);
    doc.delete_word_back().unwrap();
    assert_eq!(doc.text(), "let total = a.b(x);\r\nnext", "the indent only");
    let mut doc: Document = at(21);
    doc.delete_word_back().unwrap();
    assert_eq!(doc.text(), "let total = a.b(x);  next", "the whole \\r\\n");
    let mut doc: Document = at(3);
    doc.delete_word_forward().unwrap();
    assert_eq!(doc.text(), "let = a.b(x);\r\n  next", "` total`");
}

#[test]
fn option_backspace_with_a_selection_removes_only_the_selection() {
    let mut doc: Document = at(4);
    doc.right(true).unwrap();
    doc.delete_word_back().unwrap();
    assert_eq!(doc.text(), "let otal = a.b(x);\r\n  next");
}
