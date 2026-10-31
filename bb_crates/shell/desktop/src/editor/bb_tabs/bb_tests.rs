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
    assert_eq!(tab.parsed_text, tab.document.text());
    assert!(!tab.tree.root_node().has_error());
    tab.document.undo().unwrap();
    tab.reparse().unwrap();
    assert_eq!(tab.parsed_text, "class Café {}\n");
    assert!(!tab.tree.root_node().has_error());
}
