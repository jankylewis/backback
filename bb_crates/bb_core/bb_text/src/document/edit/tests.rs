use super::super::Document;
use super::{CharEdit, TextChange};
use crate::{Buffer, CharIdx};
use proptest::prelude::*;

fn replay(start: &str, edits: &[CharEdit]) -> String {
    let mut mirror: Buffer = Buffer::from_text(start);
    for edit in edits {
        mirror.remove(edit.from, edit.to).unwrap();
        mirror.insert(edit.from, &edit.text).unwrap();
    }
    mirror.to_text()
}

fn edit(from: usize, to: usize, text: &str) -> CharEdit {
    CharEdit {
        from: CharIdx::new(from),
        to: CharIdx::new(to),
        text: text.to_owned(),
    }
}

#[derive(Clone, Debug)]
enum Step {
    Type(String),
    Back,
    Enter,
    Undo,
    Redo,
    Move(usize),
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        "[a-zé😀.]{1,3}".prop_map(Step::Type),
        Just(Step::Back),
        Just(Step::Enter),
        Just(Step::Undo),
        Just(Step::Redo),
        (0usize..64).prop_map(Step::Move),
    ]
}

fn run(doc: &mut Document, step: &Step) {
    match step {
        Step::Type(text) => doc.insert(text).unwrap(),
        Step::Back => doc.backspace().unwrap(),
        Step::Enter => doc.enter().unwrap(),
        Step::Undo => doc.undo().unwrap(),
        Step::Redo => doc.redo().unwrap(),
        Step::Move(to) => doc
            .set_cursor(CharIdx::new((*to).min(doc.len_chars())))
            .unwrap(),
    }
}

proptest! {
    #[test]
    fn the_char_log_replayed_on_the_old_text_gives_the_new_text(
        start in "[a-z\r\n ]{0,24}",
        steps in prop::collection::vec(step(), 0..40),
        drain_at in 0usize..40,
    ) {
        let mut doc: Document = Document::from_text(&start, None);
        let mut mirror: String = start.clone();
        for (index, one) in steps.iter().enumerate() {
            run(&mut doc, one);
            if index == drain_at {
                mirror = replay(&mirror, &doc.take_char_edits());
            }
        }
        let bytes: Vec<TextChange> = doc.take_changes();
        let chars: Vec<CharEdit> = doc.take_char_edits();
        prop_assert_eq!(replay(&mirror, &chars), doc.text());
        prop_assert!(bytes.len() >= chars.len());
    }
}

#[test]
fn both_logs_record_every_edit_undo_and_redo() {
    let mut doc: Document = Document::from_text("ab", None);
    doc.set_cursor(CharIdx::new(2)).unwrap();
    doc.insert(".").unwrap();
    doc.undo().unwrap();
    doc.redo().unwrap();
    assert_eq!(doc.take_changes().len(), 3);
    assert_eq!(
        doc.take_char_edits(),
        vec![edit(2, 2, "."), edit(2, 3, ""), edit(2, 2, ".")]
    );
    assert_eq!(doc.take_char_edits(), Vec::<CharEdit>::new());
}

#[test]
fn replace_many_is_one_undo_step_and_keeps_the_cursor_on_its_text() {
    let mut doc: Document = Document::from_text("var a = a + b;", None);
    doc.set_cursor(CharIdx::new(13)).unwrap();
    doc.replace_many(&[edit(8, 9, "total"), edit(4, 5, "total")])
        .unwrap();
    assert_eq!(doc.text(), "var total = total + b;");
    assert_eq!(doc.cursor(), CharIdx::new(21));
    assert_eq!(replay("var a = a + b;", &doc.take_char_edits()), doc.text());
    doc.undo().unwrap();
    assert_eq!(doc.text(), "var a = a + b;");
    assert!(!doc.is_dirty());
    doc.redo().unwrap();
    assert_eq!(doc.text(), "var total = total + b;");
}

#[test]
fn replace_many_changes_nothing_when_one_range_is_bad() {
    let bad: [Vec<CharEdit>; 3] = [
        vec![edit(0, 3, "x"), edit(2, 4, "y")],
        vec![edit(3, 1, "x")],
        vec![edit(0, 1, "x"), edit(5, 99, "y")],
    ];
    for edits in bad {
        let mut doc: Document = Document::from_text("abcdef", None);
        assert!(doc.replace_many(&edits).is_err(), "{edits:?}");
        assert_eq!(doc.text(), "abcdef");
        assert_eq!(doc.take_char_edits(), Vec::<CharEdit>::new());
        assert!(!doc.is_dirty());
    }
}

#[test]
fn lsp_points_round_trip_through_utf16_columns() {
    let doc: Document = Document::from_text("a😀b\r\nc", None);
    let point: crate::LspPoint = doc.lsp_point(CharIdx::new(2)).unwrap();
    assert_eq!((point.line, point.character), (0, 3));
    assert_eq!(doc.char_of_lsp(point).unwrap(), CharIdx::new(2));
    let next: crate::LspPoint = crate::LspPoint {
        line: 1,
        character: 0,
    };
    assert_eq!(doc.char_of_lsp(next).unwrap(), CharIdx::new(5));
}

#[test]
fn a_write_locked_document_edits_in_memory_but_never_saves() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let path: std::path::PathBuf = dir.path().join("A.cs");
    std::fs::write(&path, "class A {}").unwrap();
    let mut doc: Document = Document::open(&path).unwrap();
    doc.lock_writes();
    doc.insert("x").unwrap();
    assert!(matches!(
        doc.save(),
        Err(super::super::DocumentError::WriteLocked(_))
    ));
    assert!(doc.is_dirty());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "class A {}");
}
