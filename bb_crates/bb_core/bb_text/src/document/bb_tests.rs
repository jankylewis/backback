use super::{Document, DocumentError};
use crate::CharIdx;
use proptest::prelude::*;
use std::fs;

#[test]
fn bom_and_crlf_survive_edit_and_atomic_save() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let path: std::path::PathBuf = dir.path().join("File.cs");
    fs::write(&path, b"\xef\xbb\xbfclass A\r\n{\r\n}\r\n").unwrap();
    let mut doc: Document = Document::open(&path).unwrap();
    doc.set_cursor(CharIdx::new(9)).unwrap();
    doc.insert("B").unwrap();
    doc.save().unwrap();
    assert_eq!(
        fs::read(&path).unwrap(),
        b"\xef\xbb\xbfclass A\r\nB{\r\n}\r\n"
    );
    assert!(!doc.is_dirty());
}

#[test]
fn consecutive_word_chars_undo_as_one_group() {
    let mut doc: Document = Document::from_text("", None);
    for letter in ["h", "e", "l", "l", "o"] {
        doc.insert(letter).unwrap();
    }
    assert_eq!(doc.text(), "hello");
    doc.undo().unwrap();
    assert_eq!(doc.text(), "");
    doc.redo().unwrap();
    assert_eq!(doc.text(), "hello");
}

#[test]
fn selection_replaces_text_and_undo_restores_it() {
    let mut doc: Document = Document::from_text("class Old {}", None);
    doc.select(CharIdx::new(6), CharIdx::new(9)).unwrap();
    doc.insert("New").unwrap();
    assert_eq!(doc.text(), "class New {}");
    doc.undo().unwrap();
    assert_eq!(doc.text(), "class Old {}");
}

#[test]
fn save_without_path_returns_typed_error() {
    let mut doc: Document = Document::from_text("x", None);
    assert!(matches!(doc.save(), Err(DocumentError::NoPath)));
}

#[test]
fn enter_after_unicode_keeps_indent_and_cursor_valid() {
    let mut doc: Document = Document::from_text("    é", None);
    doc.set_cursor(CharIdx::new(5)).unwrap();
    doc.enter().unwrap();
    assert_eq!(doc.text(), "    é\n    ");
}

#[test]
fn down_to_shorter_line_stops_before_its_line_break() {
    let mut doc: Document = Document::from_text("abcdef\r\nx\r\nnext", None);
    doc.set_cursor(CharIdx::new(5)).unwrap();
    doc.vertical(true, false).unwrap();
    assert_eq!(doc.cursor(), CharIdx::new(9));
}

proptest! {
    #[test]
    fn utf8_roundtrip_preserves_any_text(text in ".{0,100}") {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let path: std::path::PathBuf = dir.path().join("File.cs");
        fs::write(&path, text.as_bytes()).unwrap();
        let mut doc: Document = Document::open(&path).unwrap();
        doc.save().unwrap();
        prop_assert_eq!(fs::read(&path).unwrap(), text.as_bytes());
    }
}
