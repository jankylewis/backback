use std::path::PathBuf;

use super::super::{Side, Tab, TabEvent, Tabs};

/// Tabs on `names` in `dir`, opened in order (the last active).
fn open(dir: &tempfile::TempDir, names: &[&str]) -> Tabs {
    let mut tabs: Tabs = Tabs::new();
    for name in names {
        let path: PathBuf = dir.path().join(name);
        std::fs::write(&path, "class X {}").unwrap();
        tabs.open(&path).unwrap();
    }
    let _opened: Vec<TabEvent> = tabs.take_events();
    tabs
}

/// Names in list order: `>` marks the right group, `*` a pin, `[]` each
/// group's active tab, and `!` the focused group's.
fn shown(tabs: &Tabs) -> Vec<String> {
    tabs.items()
        .iter()
        .enumerate()
        .map(|(index, tab): (usize, &Tab)| {
            let mut name: String = tab.path.file_name().unwrap().to_string_lossy().into_owned();
            if tab.pinned {
                name.push('*');
            }
            if tabs.active_of(tab.side) == Some(index) {
                name = format!("[{name}]");
                if tab.side == tabs.focused() {
                    name.push('!');
                }
            }
            if tab.side == Side::Right {
                name = format!(">{name}");
            }
            name
        })
        .collect()
}

#[test]
fn a_move_right_splits_focuses_the_new_group_and_leaves_the_neighbour() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let mut tabs: Tabs = open(&dir, &["a.cs", "b.cs", "c.cs"]);
    tabs.move_to(2, Side::Right);
    assert!(tabs.is_split());
    assert_eq!(shown(&tabs), ["a.cs", "[b.cs]", ">[c.cs]!"]);
    assert_eq!(
        tabs.take_events(),
        Vec::<TabEvent>::new(),
        "a move is no close or open"
    );
    tabs.switch(0);
    assert_eq!(shown(&tabs), ["[a.cs]!", "b.cs", ">[c.cs]"]);
    tabs.switch_nth(1);
    assert_eq!(
        shown(&tabs),
        ["a.cs", "[b.cs]!", ">[c.cs]"],
        "Cmd+2 is the focused group's 2nd"
    );
}

#[test]
fn a_file_opens_once_and_new_files_open_in_the_focused_group() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let mut tabs: Tabs = open(&dir, &["a.cs", "b.cs"]);
    tabs.move_to(1, Side::Right);
    tabs.switch(0);
    tabs.open(&dir.path().join("b.cs")).unwrap();
    assert_eq!(
        shown(&tabs),
        ["[a.cs]", ">[b.cs]!"],
        "shown in its own group"
    );
    let path: PathBuf = dir.path().join("d.cs");
    std::fs::write(&path, "class D {}").unwrap();
    tabs.open(&path).unwrap();
    assert_eq!(shown(&tabs), ["[a.cs]", ">b.cs", ">[d.cs]!"]);
}

#[test]
fn closing_a_groups_last_tab_or_moving_it_away_ends_the_split() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let mut tabs: Tabs = open(&dir, &["a.cs", "b.cs", "c.cs"]);
    tabs.move_to(2, Side::Right);
    assert!(tabs.close(2));
    assert!(!tabs.is_split());
    assert_eq!(shown(&tabs), ["a.cs", "[b.cs]!"]);
    tabs.move_to(1, Side::Right);
    tabs.move_to(0, Side::Right);
    assert!(!tabs.is_split(), "every tab moved right: one group again");
    assert_eq!(shown(&tabs), ["b.cs", "[a.cs]!"]);
}

#[test]
fn bulk_closes_and_pins_stay_inside_their_group() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let mut tabs: Tabs = open(&dir, &["a.cs", "b.cs", "c.cs", "d.cs"]);
    tabs.move_to(3, Side::Right);
    tabs.move_to(2, Side::Right);
    assert_eq!(shown(&tabs), ["a.cs", "[b.cs]", ">d.cs", ">[c.cs]!"]);
    tabs.toggle_pin(3);
    assert_eq!(shown(&tabs), ["a.cs", "[b.cs]", ">[c.cs*]!", ">d.cs"]);
    assert!(tabs.close_set(2, super::super::bar::CloseSet::Unpinned));
    assert_eq!(
        shown(&tabs),
        ["a.cs", "[b.cs]", ">[c.cs*]!"],
        "the left group is untouched"
    );
    assert!(tabs.close_set(0, super::super::bar::CloseSet::All));
    assert_eq!(shown(&tabs), ["[c.cs*]!"]);
}

#[test]
fn the_save_prompt_keeps_its_file_while_tabs_move() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let mut tabs: Tabs = open(&dir, &["a.cs", "b.cs", "c.cs"]);
    tabs.switch(0);
    tabs.active_mut().unwrap().insert("x").unwrap();
    assert!(!tabs.close(0), "a.cs has an edit: asked");
    // While asked, c.cs is pinned (to the front) and b.cs moves right:
    // a.cs is no longer at its old place.
    tabs.toggle_pin(2);
    tabs.move_to(2, Side::Right);
    assert_eq!(shown(&tabs), ["c.cs*", "[a.cs]", ">[b.cs]!"]);
    tabs.discard_close();
    assert_eq!(
        shown(&tabs),
        ["[c.cs*]", ">[b.cs]!"],
        "a.cs closed, not c.cs"
    );
}
