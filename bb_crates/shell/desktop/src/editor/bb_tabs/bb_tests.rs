use super::Tabs;
use std::fs;

#[test]
fn open_deduplicates_paths_and_switches_active_tab() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let first: std::path::PathBuf = dir.path().join("A.cs");
    let second: std::path::PathBuf = dir.path().join("B.cs");
    fs::write(&first, "class A {}").unwrap();
    fs::write(&second, "class B {}").unwrap();
    let mut tabs: Tabs = Tabs::new();
    tabs.open(&first).unwrap();
    tabs.open(&second).unwrap();
    tabs.open(&first).unwrap();
    assert_eq!(tabs.len(), 2);
    assert_eq!(tabs.active_index(), Some(0));
    tabs.switch(1);
    assert_eq!(tabs.active_index(), Some(1));
}

#[test]
fn dirty_close_requires_explicit_choice() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let path: std::path::PathBuf = dir.path().join("A.cs");
    fs::write(&path, "class A {}").unwrap();
    let mut tabs: Tabs = Tabs::new();
    tabs.open(&path).unwrap();
    tabs.active_mut().unwrap().insert("x").unwrap();
    assert!(!tabs.close_active());
    assert!(tabs.pending_close());
    tabs.cancel_close();
    assert_eq!(tabs.len(), 1);
    assert!(!tabs.close_active());
    tabs.discard_close();
    assert_eq!(tabs.len(), 0);
}

#[test]
fn window_close_waits_for_each_dirty_tab() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let first: std::path::PathBuf = dir.path().join("A.cs");
    let second: std::path::PathBuf = dir.path().join("B.cs");
    fs::write(&first, "a").unwrap();
    fs::write(&second, "b").unwrap();
    let mut tabs: Tabs = Tabs::new();
    tabs.open(&first).unwrap();
    tabs.active_mut().unwrap().insert("x").unwrap();
    tabs.open(&second).unwrap();
    tabs.active_mut().unwrap().insert("y").unwrap();
    assert!(!tabs.request_window_close());
    tabs.discard_close();
    assert!(!tabs.request_window_close());
    tabs.discard_close();
    assert!(tabs.request_window_close());
}

fn fresh_sexp(tab: &super::Tab) -> String {
    let mut syntax: bb_syntax::CSharpSyntax = bb_syntax::CSharpSyntax::new().unwrap();
    let tree: bb_syntax::Tree = syntax.parse(&tab.document.text(), None).unwrap();
    tree.root_node().to_sexp()
}

#[test]
fn incremental_reparse_tracks_unicode_insert_and_undo() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let path: std::path::PathBuf = dir.path().join("Unicode.cs");
    fs::write(&path, "class Café {}\n").unwrap();
    let mut tabs: Tabs = Tabs::new();
    tabs.open(&path).unwrap();
    let tab: &mut super::Tab = tabs.active_tab_mut().unwrap();
    tab.document.set_cursor(bb_text::CharIdx::new(10)).unwrap();
    tab.document.insert("界").unwrap();
    tab.reparse().unwrap();
    assert_eq!(tab.live.tree().root_node().to_sexp(), fresh_sexp(tab));
    assert_eq!(
        tab.live.tree().root_node().end_byte(),
        tab.document.rope().len_bytes()
    );
    tab.document.undo().unwrap();
    tab.reparse().unwrap();
    assert_eq!(tab.document.text(), "class Café {}\n");
    assert_eq!(tab.live.tree().root_node().to_sexp(), fresh_sexp(tab));
    assert!(!tab.live.tree().root_node().has_error());
}

#[test]
fn a_loading_file_shows_once_and_a_failed_load_clears_its_placeholder() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let path: std::path::PathBuf = dir.path().join("Latin.cs");
    fs::write(&path, b"class \xe9 {}").unwrap();
    let mut tabs: Tabs = Tabs::new();
    let file: std::path::PathBuf = tabs.begin_open(&path).unwrap().unwrap();
    assert!(tabs.begin_open(&path).unwrap().is_none());
    assert_eq!(tabs.loading(), std::slice::from_ref(&file));
    let loaded: Result<super::Tab, super::TabError> = super::Tab::load(file.clone());
    assert!(tabs.finish_open(&file, loaded).is_err());
    assert_eq!(tabs.loading().len(), 0);
    assert_eq!(tabs.len(), 0);
}
