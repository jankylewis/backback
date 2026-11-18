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
fn metadata_is_hidden_and_gitignored_entries_are_dimmed() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    fs::create_dir(dir.path().join("Nested")).unwrap();
    fs::write(dir.path().join(".DS_Store"), "").unwrap();
    fs::write(dir.path().join(".gitignore"), "*.tmp\nNested/*.log\n").unwrap();
    fs::write(dir.path().join("scratch.tmp"), "").unwrap();
    fs::write(dir.path().join("keep.cs"), "").unwrap();
    fs::write(dir.path().join("Nested/trace.log"), "").unwrap();
    let mut tree: Explorer = Explorer::open(dir.path()).unwrap();
    let root_names: Vec<&str> = tree
        .root()
        .children()
        .iter()
        .map(crate::Entry::name)
        .collect();
    assert!(!root_names.contains(&".DS_Store"));
    assert!(!root_names.contains(&".git"));
    let scratch: &crate::Entry = tree
        .root()
        .children()
        .iter()
        .find(|entry: &&crate::Entry| entry.name() == "scratch.tmp")
        .unwrap();
    assert!(scratch.dimmed());
    let keep: &crate::Entry = tree
        .root()
        .children()
        .iter()
        .find(|entry: &&crate::Entry| entry.name() == "keep.cs")
        .unwrap();
    assert!(!keep.dimmed());
    tree.toggle(&dir.path().join("Nested")).unwrap();
    assert!(tree
        .root()
        .children()
        .iter()
        .find(|entry: &&crate::Entry| entry.name() == "Nested")
        .unwrap()
        .children()
        .iter()
        .find(|entry: &&crate::Entry| entry.name() == "trace.log")
        .unwrap()
        .dimmed());
}

#[test]
fn nested_gitignore_negation_controls_dim_state() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("Nested")).unwrap();
    fs::write(dir.path().join(".gitignore"), "*.log\n").unwrap();
    fs::write(dir.path().join("Nested/.gitignore"), "!important.log\n").unwrap();
    fs::write(dir.path().join("Nested/trace.log"), "").unwrap();
    fs::write(dir.path().join("Nested/important.log"), "").unwrap();
    let mut tree: Explorer = Explorer::open(dir.path()).unwrap();
    tree.toggle(&dir.path().join("Nested")).unwrap();
    let nested: &crate::Entry = tree
        .root()
        .children()
        .iter()
        .find(|entry: &&crate::Entry| entry.name() == "Nested")
        .unwrap();
    let trace: &crate::Entry = nested
        .children()
        .iter()
        .find(|entry: &&crate::Entry| entry.name() == "trace.log")
        .unwrap();
    let important: &crate::Entry = nested
        .children()
        .iter()
        .find(|entry: &&crate::Entry| entry.name() == "important.log")
        .unwrap();
    assert!(trace.dimmed());
    assert!(!important.dimmed());
}

#[test]
fn opening_subfolder_keeps_parent_gitignore_rules() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("Nested/Child")).unwrap();
    fs::write(dir.path().join(".gitignore"), "*.tmp\n").unwrap();
    fs::write(dir.path().join("Nested/Child/scratch.tmp"), "").unwrap();
    let tree: Explorer = Explorer::open(&dir.path().join("Nested/Child")).unwrap();
    assert!(tree.root().children().first().unwrap().dimmed());
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

#[test]
fn files_sort_by_number_value_not_by_text() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    for name in [
        "f10.cs",
        "f2.cs",
        "F1.cs",
        "f100.cs",
        "f1001.cs",
        "f10000.cs",
        "f02.cs",
    ] {
        fs::write(dir.path().join(name), "").unwrap();
    }
    let tree: Explorer = Explorer::open(dir.path()).unwrap();
    let names: Vec<&str> = tree
        .root()
        .children()
        .iter()
        .map(crate::Entry::name)
        .collect();
    let expected: &str = "F1.cs f02.cs f2.cs f10.cs f100.cs f1001.cs f10000.cs";
    assert_eq!(names.join(" "), expected);
}

proptest! {
    #[test]
    fn name_order_is_a_total_order(
        a in "[a-zA-Z0-9]{1,12}",
        b in "[a-zA-Z0-9]{1,12}",
        c in "[a-zA-Z0-9]{1,12}",
    ) {
        use std::cmp::Ordering;
        let order: fn(&str, &str) -> Ordering = crate::bb_entry::name_order;
        prop_assert_eq!(order(&a, &b), order(&b, &a).reverse());
        prop_assert_eq!(order(&a, &b) == Ordering::Equal, a == b);
        if order(&a, &b) != Ordering::Greater && order(&b, &c) != Ordering::Greater {
            prop_assert_ne!(order(&a, &c), Ordering::Greater);
        }
    }

    #[test]
    fn case_insensitive_sort_is_stable(a in "[a-zA-Z]{1,12}", b in "[a-zA-Z]{1,12}") {
        let comparison: std::cmp::Ordering = crate::bb_entry::name_order(&a, &b);
        let reverse: std::cmp::Ordering = crate::bb_entry::name_order(&b, &a);
        prop_assert_eq!(comparison, reverse.reverse());
        if a.eq_ignore_ascii_case(&b) {
            prop_assert_eq!(comparison, a.cmp(&b));
        }
    }
}
