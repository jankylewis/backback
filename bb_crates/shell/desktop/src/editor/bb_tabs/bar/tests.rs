use std::path::PathBuf;

use super::super::{Tab, Tabs};
use super::CloseSet;

/// Tabs on files `a.cs`, `b.cs`, ... in `dir`, the last one active.
fn open(dir: &tempfile::TempDir, names: &[&str]) -> Tabs {
    let mut tabs: Tabs = Tabs::new();
    for name in names {
        let path: PathBuf = dir.path().join(name);
        std::fs::write(&path, "class X {}").unwrap();
        tabs.open(&path).unwrap();
    }
    tabs
}

fn names(tabs: &Tabs) -> Vec<String> {
    tabs.items()
        .iter()
        .map(|tab: &Tab| {
            let name: String = tab.path.file_name().unwrap().to_string_lossy().into_owned();
            if tab.pinned {
                format!("{name}*")
            } else {
                name
            }
        })
        .collect()
}

fn active(tabs: &Tabs) -> String {
    names(tabs)
        .get(tabs.active_index().unwrap())
        .unwrap()
        .clone()
}

#[test]
fn a_pin_moves_the_tab_left_and_an_unpin_after_the_pinned_run() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let mut tabs: Tabs = open(&dir, &["a.cs", "b.cs", "c.cs", "d.cs"]);
    tabs.toggle_pin(2);
    assert_eq!(names(&tabs), ["c.cs*", "a.cs", "b.cs", "d.cs"]);
    assert_eq!(active(&tabs), "d.cs", "the active file stays active");
    tabs.toggle_pin(3);
    assert_eq!(names(&tabs), ["c.cs*", "d.cs*", "a.cs", "b.cs"]);
    tabs.toggle_pin(0);
    assert_eq!(names(&tabs), ["d.cs*", "c.cs", "a.cs", "b.cs"]);
    assert_eq!(active(&tabs), "d.cs*");
}

#[test]
fn others_right_and_left_skip_pinned_tabs() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let all: [&str; 5] = ["a.cs", "b.cs", "c.cs", "d.cs", "e.cs"];
    let mut tabs: Tabs = open(&dir, &all);
    tabs.toggle_pin(4);
    // e* a b c d: close to the right of b.
    assert!(tabs.close_set(2, CloseSet::Right));
    assert_eq!(names(&tabs), ["e.cs*", "a.cs", "b.cs"]);
    assert_eq!(active(&tabs), "b.cs", "the menu's tab becomes active");
    assert!(tabs.close_set(2, CloseSet::Left));
    assert_eq!(names(&tabs), ["e.cs*", "b.cs"]);
    let mut tabs: Tabs = open(&dir, &all);
    tabs.toggle_pin(0);
    assert!(tabs.close_set(2, CloseSet::Others));
    assert_eq!(names(&tabs), ["a.cs*", "c.cs"]);
}

#[test]
fn close_all_except_pinned_keeps_pins_and_close_all_takes_every_tab() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let mut tabs: Tabs = open(&dir, &["a.cs", "b.cs", "c.cs"]);
    tabs.toggle_pin(1);
    assert!(tabs.close_set(0, CloseSet::Unpinned));
    assert_eq!(names(&tabs), ["b.cs*"]);
    assert_eq!(tabs.active_index(), Some(0));
    assert!(tabs.close_set(0, CloseSet::All));
    assert_eq!(names(&tabs), Vec::<String>::new());
    assert_eq!(tabs.active_index(), None);
}

#[test]
fn a_bulk_close_asks_about_each_dirty_tab_in_turn_and_cancel_stops_it() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let mut tabs: Tabs = open(&dir, &["a.cs", "b.cs", "c.cs", "d.cs"]);
    for index in [0, 2] {
        tabs.switch(index);
        tabs.active_mut().unwrap().insert("x").unwrap();
    }
    assert!(!tabs.close_set(3, CloseSet::All), "a and c have edits");
    assert_eq!(names(&tabs), ["a.cs", "c.cs"], "clean tabs closed at once");
    assert!(tabs.pending_close());
    assert_eq!(active(&tabs), "a.cs", "the asked-about tab is shown");
    tabs.discard_close();
    assert!(tabs.pending_close(), "then c is asked about");
    assert_eq!(active(&tabs), "c.cs");
    tabs.discard_close();
    assert!(!tabs.pending_close());
    assert_eq!(names(&tabs), Vec::<String>::new());
    let mut tabs: Tabs = open(&dir, &["a.cs", "b.cs", "c.cs"]);
    for index in [0, 1] {
        tabs.switch(index);
        tabs.active_mut().unwrap().insert("x").unwrap();
    }
    assert!(!tabs.close_set(2, CloseSet::Others));
    tabs.cancel_close();
    assert!(!tabs.pending_close(), "cancel ends the whole close");
    assert_eq!(names(&tabs), ["a.cs", "b.cs", "c.cs"]);
}
