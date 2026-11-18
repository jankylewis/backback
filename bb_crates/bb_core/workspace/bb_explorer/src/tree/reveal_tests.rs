//! BB-BUG-029: the Explorer follows the active file.

use std::fs;
use std::path::{Path, PathBuf};

use crate::Explorer;

fn names(tree: &Explorer) -> Vec<String> {
    tree.visible()
        .iter()
        .map(|(depth, entry): &(usize, &crate::Entry)| format!("{depth}:{}", entry.name()))
        .collect()
}

#[test]
fn reveal_opens_every_folder_down_to_the_file_and_gives_its_row() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = dir.path().canonicalize().unwrap();
    fs::create_dir_all(root.join("MbAx/agentmates/autogen")).unwrap();
    fs::create_dir_all(root.join("Other")).unwrap();
    let file: PathBuf = root.join("MbAx/agentmates/autogen/walk.py");
    fs::write(&file, "x = 1\n").unwrap();
    fs::write(root.join("README.md"), "").unwrap();
    let mut tree: Explorer = Explorer::open(&root).unwrap();
    let row: Option<usize> = tree.reveal(&file).unwrap();
    let rows: Vec<String> = names(&tree);
    assert_eq!(
        rows,
        [
            "0:".to_owned() + tree.root().name(),
            "1:MbAx".into(),
            "2:agentmates".into(),
            "3:autogen".into(),
            "4:walk.py".into(),
            "1:Other".into(),
            "1:README.md".into()
        ]
    );
    assert_eq!(row, Some(4));
    assert_eq!(
        tree.reveal(&file).unwrap(),
        Some(4),
        "a second reveal is the same row"
    );
}

#[test]
fn a_file_outside_the_tree_reveals_nothing() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let mut tree: Explorer = Explorer::open(dir.path()).unwrap();
    assert_eq!(tree.reveal(Path::new("/nowhere/a.cs")).unwrap(), None);
}

/// BB-BUG-037: files made and removed outside show after a refresh; an
/// open folder stays open.
#[test]
fn refresh_takes_outside_changes_and_keeps_open_folders_open() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = dir.path().canonicalize().unwrap();
    fs::create_dir_all(root.join("sub")).unwrap();
    fs::write(root.join("sub/b.txt"), "").unwrap();
    fs::write(root.join("a.py"), "").unwrap();
    let mut tree: Explorer = Explorer::open(&root).unwrap();
    tree.reveal(&root.join("sub/b.txt")).unwrap();
    assert!(!tree.refresh().unwrap(), "nothing changed");
    fs::write(root.join("sub/new.py"), "").unwrap();
    fs::remove_file(root.join("a.py")).unwrap();
    assert!(tree.refresh().unwrap());
    let rows: Vec<String> = names(&tree);
    assert!(rows.contains(&"2:new.py".to_owned()), "{rows:?}");
    assert!(
        !rows.iter().any(|row: &String| row.ends_with(":a.py")),
        "{rows:?}"
    );
    assert!(
        rows.contains(&"2:b.txt".to_owned()),
        "sub stayed open: {rows:?}"
    );
}
