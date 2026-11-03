use std::fs;
use std::path::PathBuf;

use super::super::{Document, DocumentError};

/// BB-BUG-034: a save over a file another tool changed is refused once;
/// the next save overwrites; saves of our own changes never trip it.
#[test]
fn a_save_over_an_outside_change_is_refused_once_then_overwrites() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let file: PathBuf = dir.path().join("A.cs");
    fs::write(&file, "class A\n{\n}\n").unwrap();
    let mut doc: Document = Document::open(&file).unwrap();
    doc.insert("x").unwrap();
    doc.save().unwrap();
    doc.insert("y").unwrap();
    doc.save().unwrap();
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        "xyclass A\n{\n}\n",
        "our own saves pass"
    );
    fs::write(&file, "class A\n{\n    // another tool\n}\n").unwrap();
    doc.insert("z").unwrap();
    let refused: Result<(), DocumentError> = doc.save();
    assert!(
        matches!(refused, Err(DocumentError::ChangedOnDisk(_))),
        "{refused:?}"
    );
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        "class A\n{\n    // another tool\n}\n",
        "kept"
    );
    doc.save().unwrap();
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        "xyzclass A\n{\n}\n",
        "the second save overwrites"
    );
    doc.insert("w").unwrap();
    doc.save().unwrap();
}

/// BB-BUG-035: a clean buffer takes the disk's text; a dirty one never does.
#[test]
fn a_clean_buffer_takes_an_outside_change_and_a_dirty_one_keeps_its_text() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let file: PathBuf = dir.path().join("A.cs");
    fs::write(&file, "class A\n{\n}\n").unwrap();
    let mut doc: Document = Document::open(&file).unwrap();
    assert!(!doc.reload_if_changed().unwrap(), "no change, no reload");
    fs::write(&file, "class B\n{\n    int x;\n}\n").unwrap();
    assert!(doc.reload_if_changed().unwrap());
    assert_eq!(doc.text(), "class B\n{\n    int x;\n}\n");
    assert!(!doc.is_dirty(), "the disk's text is the saved text");
    doc.undo().unwrap();
    assert_eq!(
        doc.text(),
        "class A\n{\n}\n",
        "one undo step brings back the old text"
    );
    doc.redo().unwrap();
    doc.insert("y").unwrap();
    fs::write(&file, "class C { }\n").unwrap();
    assert!(
        !doc.reload_if_changed().unwrap(),
        "unsaved edits are never thrown away"
    );
    assert!(doc.text().starts_with('y'));
}
