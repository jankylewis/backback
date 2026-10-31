use std::{fs, path::Path};

use proptest::prelude::*;

use crate::{EntryKind, Explorer};

#[test]
fn folder_is_sorted_lazily_and_classified() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("obj")).unwrap();
    fs::create_dir(dir.path().join("Alpha")).unwrap();
    fs::write(dir.path().join("z.cs"), "").unwrap();
    fs::write(dir.path().join("B.md"), "").unwrap();
    let mut tree: Explorer = Explorer::open(dir.path()).unwrap();
    let names: Vec<String> = tree
        .root()
        .children()
        .iter()
        .map(|entry: &crate::Entry| entry.name().to_owned())
        .collect();
    assert_eq!(names, ["Alpha", "obj", "B.md", "z.cs"]);
    let obj: &crate::Entry = tree.root().children().get(1).unwrap();
    assert!(obj.dimmed());
    assert_eq!(obj.kind(), EntryKind::Folder);
    assert_eq!(
        tree.root().children().get(2).unwrap().kind(),
        EntryKind::Markdown
    );
    assert!(!tree.root().children().first().unwrap().loaded());
    tree.toggle(&dir.path().join("Alpha")).unwrap();
    assert!(tree.root().children().first().unwrap().loaded());
}

#[test]
fn slnx_shows_only_projects_and_their_folders() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("Core")).unwrap();
    fs::write(dir.path().join("Core/Core.csproj"), "").unwrap();
    fs::write(dir.path().join("Unused.csproj"), "").unwrap();
    fs::write(
        dir.path().join("App.slnx"),
        "<Solution><Project Path=\"Core/Core.csproj\" /></Solution>",
    )
    .unwrap();
    let mut tree: Explorer = Explorer::open(&dir.path().join("App.slnx")).unwrap();
    assert_eq!(tree.root().kind(), EntryKind::Solution);
    assert_eq!(tree.root().children().len(), 1);
    tree.toggle(&dir.path().join("Core")).unwrap();
    assert_eq!(
        tree.root()
            .children()
            .first()
            .unwrap()
            .children()
            .first()
            .unwrap()
            .kind(),
        EntryKind::Project
    );
}

#[test]
fn entry_paths_are_full_tooltip_paths() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("A.cs"), "").unwrap();
    let tree: Explorer = Explorer::open(dir.path()).unwrap();
    assert_eq!(
        tree.root().children().first().unwrap().path(),
        dir.path().join("A.cs")
    );
    assert!(Path::new(tree.root().children().first().unwrap().path()).is_absolute());
}

#[test]
fn filesystem_root_has_a_visible_name() {
    let tree: Explorer = Explorer::open(Path::new("/")).unwrap();
    assert_eq!(tree.root().name(), "/");
}

#[test]
fn nested_solution_file_is_not_expandable() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let file: std::path::PathBuf = dir.path().join("Harness.slnx");
    fs::write(&file, "<Solution />").unwrap();
    let mut tree: Explorer = Explorer::open(dir.path()).unwrap();
    let entry: &crate::Entry = tree.root().children().first().unwrap();
    assert_eq!(entry.kind(), EntryKind::Solution);
    assert!(!entry.expandable());
    tree.toggle(&file).unwrap();
}

#[test]
fn expand_all_reveals_nested_folders_once() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("Core/Nested")).unwrap();
    fs::write(dir.path().join("Core/Nested/Foo.cs"), "").unwrap();
    let mut tree: Explorer = Explorer::open(dir.path()).unwrap();
    assert_eq!(tree.visible().len(), 2);
    assert_eq!(tree.expand_all().unwrap(), 2);
    assert_eq!(tree.visible().len(), 4);
    assert_eq!(tree.expand_all().unwrap(), 2);
}

proptest! {
    #[test]
    fn case_insensitive_sort_is_stable(a in "[a-zA-Z]{1,12}", b in "[a-zA-Z]{1,12}") {
        let comparison: std::cmp::Ordering = crate::entry::name_order(&a, &b);
        let reverse: std::cmp::Ordering = crate::entry::name_order(&b, &a);
        prop_assert_eq!(comparison, reverse.reverse());
        if a.eq_ignore_ascii_case(&b) {
            prop_assert_eq!(comparison, a.cmp(&b));
        }
    }
}
