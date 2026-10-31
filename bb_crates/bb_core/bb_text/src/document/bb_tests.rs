use super::{BytePoint, Document, DocumentError, TextChange};
use crate::CharIdx;
use proptest::prelude::*;
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

#[test]
fn bom_and_crlf_survive_edit_and_atomic_save() {
    let (_dir, path): (tempfile::TempDir, PathBuf) =
        fixture("File.cs", "\u{feff}class A\r\n{\r\n}\r\n");
    let mut doc: Document = Document::open(&path).unwrap();
    doc.set_cursor(CharIdx::new(9)).unwrap();
    doc.insert("B").unwrap();
    doc.save().unwrap();
    let expected: &[u8] = b"\xef\xbb\xbfclass A\r\nB{\r\n}\r\n";
    assert_eq!(fs::read(&path).unwrap(), expected);
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

fn edited(path: &Path) -> Document {
    let mut doc: Document = Document::open(path).unwrap();
    doc.set_cursor(CharIdx::new(0)).unwrap();
    doc.insert("// edit\n").unwrap();
    doc
}

fn fixture(name: &str, text: &str) -> (tempfile::TempDir, PathBuf) {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let path: PathBuf = dir.path().join(name);
    fs::write(&path, text).unwrap();
    (dir, path)
}

#[test]
fn read_only_file_save_fails_typed_and_keeps_bytes_and_inode() {
    let (_dir, path): (tempfile::TempDir, PathBuf) = fixture("Locked.cs", "class Locked {}\n");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o444)).unwrap();
    let inode: u64 = fs::metadata(&path).unwrap().ino();
    let mut doc: Document = edited(&path);
    let outcome: Result<(), DocumentError> = doc.save();
    assert!(
        matches!(outcome, Err(DocumentError::ReadOnly(_))),
        "{outcome:?}"
    );
    assert_eq!(fs::read(&path).unwrap(), b"class Locked {}\n");
    assert_eq!(fs::metadata(&path).unwrap().ino(), inode);
    assert_eq!(fs::metadata(&path).unwrap().mode() & 0o777, 0o444);
    assert!(doc.is_dirty());
}

#[test]
fn hard_link_keeps_inode_and_both_names_see_the_save() {
    let (dir, path): (tempfile::TempDir, PathBuf) = fixture("Linked.cs", "class L {}\n");
    let other: PathBuf = dir.path().join("Other.cs");
    fs::hard_link(&path, &other).unwrap();
    let inode: u64 = fs::metadata(&path).unwrap().ino();
    edited(&path).save().unwrap();
    assert_eq!(fs::metadata(&path).unwrap().ino(), inode);
    assert_eq!(fs::metadata(&path).unwrap().nlink(), 2);
    assert_eq!(fs::read(&other).unwrap(), b"// edit\nclass L {}\n");
}

#[test]
fn mode_and_extended_attribute_survive_save() {
    let (_dir, path): (tempfile::TempDir, PathBuf) = fixture("Tagged.cs", "class T {}\n");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
    let wrote: std::process::ExitStatus = std::process::Command::new("xattr")
        .args(["-w", "user.backback", "kept"])
        .arg(&path)
        .status()
        .unwrap();
    assert!(wrote.success());
    edited(&path).save().unwrap();
    assert_eq!(fs::metadata(&path).unwrap().mode() & 0o777, 0o640);
    let read: std::process::Output = std::process::Command::new("xattr")
        .args(["-p", "user.backback"])
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&read.stdout).trim(), "kept");
    assert_eq!(fs::read(&path).unwrap(), b"// edit\nclass T {}\n");
}

#[test]
fn symlink_path_save_writes_target_and_keeps_link() {
    let (dir, target): (tempfile::TempDir, PathBuf) = fixture("Target.cs", "class S {}\n");
    let link: PathBuf = dir.path().join("Link.cs");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    edited(&link).save().unwrap();
    assert!(fs::symlink_metadata(&link)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(fs::read(&target).unwrap(), b"// edit\nclass S {}\n");
}

#[test]
fn read_only_folder_save_fails_and_keeps_bytes() {
    let (dir, path): (tempfile::TempDir, PathBuf) = fixture("InLocked.cs", "class D {}\n");
    let mut doc: Document = edited(&path);
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o555)).unwrap();
    let outcome: Result<(), DocumentError> = doc.save();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(outcome.is_err());
    assert_eq!(fs::read(&path).unwrap(), b"class D {}\n");
    assert!(doc.is_dirty());
}

#[test]
fn dangling_symlink_save_creates_target_and_keeps_link() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let link: PathBuf = dir.path().join("Link.cs");
    std::os::unix::fs::symlink("Made.cs", &link).unwrap();
    let mut doc: Document = Document::from_text("class M {}\n", Some(link.clone()));
    doc.save().unwrap();
    assert!(fs::symlink_metadata(&link)
        .unwrap()
        .file_type()
        .is_symlink());
    let made: Vec<u8> = fs::read(dir.path().join("Made.cs")).unwrap();
    assert_eq!(made, b"class M {}\n");
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

fn point_in(text: &str, byte: usize) -> BytePoint {
    let prefix: &str = text.get(..byte).unwrap();
    let row: usize = prefix.matches('\n').count();
    let column: usize = prefix.rfind('\n').map_or(byte, |nl: usize| byte - nl - 1);
    BytePoint { byte, row, column }
}

fn apply(doc: &mut Document, op: u8, arg: usize) {
    let at: CharIdx = CharIdx::new(arg % (doc.len_chars() + 1));
    match op % 7 {
        3 => doc.select(CharIdx::new(0), at).unwrap(),
        4 | 5 => {}
        _ => doc.set_cursor(at).unwrap(),
    }
    match op % 7 {
        0 => doc.insert(["é", "\r", "\u{2028}"][arg % 3]),
        1 => doc.backspace(),
        2 => doc.enter(),
        3 => doc.insert("x\r\ny"),
        4 => doc.undo(),
        5 => doc.redo(),
        _ => doc.delete(),
    }
    .unwrap();
}

proptest! {
    #[test]
    fn every_edit_logs_its_bytes_and_clean_means_saved_text(
        start in "[a-z \n]{0,30}",
        ops in proptest::collection::vec((0u8..7, 0usize..64), 1..60),
    ) {
        let mut doc: Document = Document::from_text(&start, None);
        for (op, arg) in ops {
            let mut replay: String = doc.text();
            apply(&mut doc, op, arg);
            let new: String = doc.text();
            for change in doc.take_changes() {
                let TextChange { start, old_end, new_end }: TextChange = change;
                prop_assert_eq!(start, point_in(&replay, start.byte));
                prop_assert_eq!(old_end, point_in(&replay, old_end.byte));
                let inserted: &str = new.get(start.byte..new_end.byte).unwrap();
                replay.replace_range(start.byte..old_end.byte, inserted);
                prop_assert_eq!(new_end, point_in(&replay, new_end.byte));
            }
            prop_assert_eq!(&replay, &new);
            if !doc.is_dirty() {
                prop_assert_eq!(&new, &start);
            }
        }
    }
}

#[test]
fn undo_to_the_save_point_is_clean_and_redo_past_it_is_dirty() {
    let (_dir, path): (tempfile::TempDir, PathBuf) = fixture("Point.cs", "class P {}\n");
    let mut doc: Document = edited(&path);
    doc.save().unwrap();
    doc.insert(" ").unwrap();
    assert!(doc.is_dirty());
    doc.undo().unwrap();
    assert!(!doc.is_dirty());
    doc.undo().unwrap();
    assert!(doc.is_dirty());
    doc.redo().unwrap();
    assert!(!doc.is_dirty());
}
