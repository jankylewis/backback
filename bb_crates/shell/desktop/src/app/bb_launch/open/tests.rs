//! BBEOD23: the start page's entries, and opening one from the page.

use std::path::{Path, PathBuf};

use gpui::{AnyWindowHandle, App, TestAppContext, VisualTestContext, WindowHandle};

use super::bb_start_page::{open, StartPage};
use super::start_entries::{entries, shown_path, StartEntry};
use crate::ExplorerView;

fn folder(base: &Path, name: &str) -> PathBuf {
    let path: PathBuf = base.join(name);
    std::fs::create_dir_all(&path).unwrap();
    path.canonicalize().unwrap()
}

#[test]
fn the_entries_keep_the_recent_folders_on_disk_newest_first_then_open() {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let (new, old): (PathBuf, PathBuf) = (folder(base.path(), "New"), folder(base.path(), "Old"));
    let gone: PathBuf = base.path().join("Gone");
    assert_eq!(
        entries(vec![new.clone(), gone, old.clone()]),
        vec![
            StartEntry::Recent(new),
            StartEntry::Recent(old),
            StartEntry::Open
        ]
    );
    assert_eq!(
        entries(Vec::new()),
        vec![StartEntry::Open],
        "no recent: Open... only"
    );
}

#[test]
fn a_path_under_home_is_shown_with_a_tilde() {
    let home: &Path = Path::new("/h/me");
    assert_eq!(
        shown_path(Path::new("/h/me/src/App"), Some(home)),
        "~/src/App"
    );
    assert_eq!(shown_path(home, Some(home)), "~");
    assert_eq!(shown_path(Path::new("/opt/App"), Some(home)), "/opt/App");
}

#[gpui::test]
fn down_and_enter_open_the_second_entry_and_close_the_page(cx: &mut TestAppContext) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let (first, second): (PathBuf, PathBuf) =
        (folder(base.path(), "First"), folder(base.path(), "Second"));
    let recent: Vec<PathBuf> = vec![first, second.clone()];
    let page: WindowHandle<StartPage> = cx.update(|cx: &mut App| open(recent, false, cx)).unwrap();
    let mut vcx: VisualTestContext = VisualTestContext::from_window(page.into(), cx);
    vcx.simulate_keystrokes("down");
    let selected: usize = page
        .read_with(&vcx, |p: &StartPage, _: &App| p.selected())
        .unwrap();
    assert_eq!(selected, 1);
    vcx.simulate_keystrokes("enter");
    vcx.run_until_parked();
    let windows: Vec<AnyWindowHandle> = cx.windows();
    assert!(
        windows
            .iter()
            .all(|w: &AnyWindowHandle| w.downcast::<StartPage>().is_none()),
        "the page closed"
    );
    let roots: Vec<PathBuf> = windows
        .iter()
        .filter_map(|w: &AnyWindowHandle| w.downcast::<ExplorerView>())
        .map(|w: WindowHandle<ExplorerView>| {
            w.read_with(cx, |v: &ExplorerView, _: &App| v.root.clone())
                .unwrap()
        })
        .collect();
    assert_eq!(
        roots,
        vec![second],
        "the second entry opened in its own window"
    );
}
