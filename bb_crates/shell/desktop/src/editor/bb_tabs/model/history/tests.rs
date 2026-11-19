use std::path::{Path, PathBuf};

use bb_text::LspPoint;

use super::bb_stack::KEEP;
use super::{History, Hop, Place};

fn at(path: &Path, line: u32) -> Place {
    Place {
        path: path.to_path_buf(),
        at: LspPoint { line, character: 0 },
    }
}

fn all_open(_: &Path) -> bool {
    true
}

fn none_open(_: &Path) -> bool {
    false
}

fn went(to: Place) -> Hop {
    Hop {
        to: Some(to),
        gone: Vec::new(),
    }
}

/// A folder with `a.cs` and `b.cs` on disk.
fn two_files() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let a: PathBuf = dir.path().join("a.cs");
    let b: PathBuf = dir.path().join("b.cs");
    std::fs::write(&a, "a\n").unwrap();
    std::fs::write(&b, "b\n").unwrap();
    (dir, a, b)
}

#[test]
fn back_and_forward_retrace_the_jumps_and_a_new_jump_clears_forward() {
    let (a, b, c, d): (&Path, &Path, &Path, &Path) = (
        Path::new("/a.cs"),
        Path::new("/b.cs"),
        Path::new("/c.cs"),
        Path::new("/d.cs"),
    );
    let mut history: History = History::default();
    history.leave(at(a, 1));
    history.leave(at(b, 2));
    assert_eq!(history.back(Some(at(c, 3)), &all_open), went(at(b, 2)));
    assert_eq!(history.back(Some(at(b, 2)), &all_open), went(at(a, 1)));
    assert_eq!(history.back(Some(at(a, 1)), &all_open), Hop::default());
    assert_eq!(history.forward(Some(at(a, 1)), &all_open), went(at(b, 2)));
    history.leave(at(b, 2));
    assert_eq!(
        history.forward(Some(at(d, 4)), &all_open),
        Hop::default(),
        "a new jump clears Forward"
    );
}

#[test]
fn back_keeps_at_most_fifty_places() {
    let mut history: History = History::default();
    for line in 0..60 {
        history.leave(at(Path::new("/a.cs"), line));
    }
    let mut count: usize = 0;
    while history
        .back(Some(at(Path::new("/x.cs"), 0)), &all_open)
        .to
        .is_some()
    {
        count += 1;
    }
    assert_eq!(count, KEEP);
}

#[test]
fn with_every_tab_closed_back_still_goes_to_a_file_on_disk_and_is_live() {
    let (_dir, a, _b): (tempfile::TempDir, PathBuf, PathBuf) = two_files();
    let mut history: History = History::default();
    history.leave(at(&a, 3));
    assert!(
        history.can_back(&none_open),
        "a closed file on disk is reachable"
    );
    assert_eq!(history.back(None, &none_open), went(at(&a, 3)));
    assert!(!history.can_back(&none_open), "nothing left");
}

#[test]
fn back_skips_a_deleted_file_names_it_and_lands_on_the_next_place() {
    let (_dir, a, b): (tempfile::TempDir, PathBuf, PathBuf) = two_files();
    let mut history: History = History::default();
    history.leave(at(&a, 1));
    history.leave(at(&b, 2));
    std::fs::remove_file(&b).unwrap();
    let hop: Hop = history.back(Some(at(&a, 5)), &none_open);
    assert_eq!(hop.to, Some(at(&a, 1)), "b.cs is gone: skipped");
    assert_eq!(hop.gone, vec![b.clone()], "the skipped file is named");
    assert_eq!(
        history.forward(Some(at(&a, 1)), &none_open),
        went(at(&a, 5)),
        "Forward returns to where Back started, never to the deleted file"
    );
}

#[test]
fn the_arrows_are_off_when_every_place_is_in_a_deleted_file_unless_its_tab_is_open() {
    let (_dir, a, b): (tempfile::TempDir, PathBuf, PathBuf) = two_files();
    let mut history: History = History::default();
    history.leave(at(&b, 2));
    std::fs::remove_file(&b).unwrap();
    assert!(!history.can_back(&none_open), "← off: b.cs is deleted");
    let b_open: &dyn Fn(&Path) -> bool = &|path: &Path| path == b;
    assert!(
        history.can_back(b_open),
        "an open tab is reachable off disk"
    );
    let hop: Hop = history.back(Some(at(&a, 0)), &none_open);
    assert_eq!(hop.to, None, "no reachable place");
    assert_eq!(hop.gone, vec![b], "the press says which file is gone");
    assert!(
        !history.can_forward(&none_open),
        "a press that went nowhere puts nothing on Forward"
    );
}
