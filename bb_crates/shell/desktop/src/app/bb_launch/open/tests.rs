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
    // BB-BUG-048: titled with its folder, so the Window menu lists it.
    let opened: AnyWindowHandle = windows
        .into_iter()
        .find(|w: &AnyWindowHandle| w.downcast::<ExplorerView>().is_some())
        .unwrap();
    let mut wcx: VisualTestContext = VisualTestContext::from_window(opened, cx);
    assert_eq!(wcx.window_title().as_deref(), Some("Second"));
}

/// BB-BUG-036: Cmd+P is Go to File, as in VS Code.
#[test]
fn cmd_p_opens_go_to_file() {
    let found: bool = super::bindings().iter().any(|binding: &gpui::KeyBinding| {
        binding.keystrokes().len() == 1
            && binding
                .keystrokes()
                .first()
                .is_some_and(|key: &gpui::KeybindingKeystroke| {
                    key.inner().key == "p" && key.inner().modifiers == gpui::Modifiers::command()
                })
            && binding.action().name() == gpui::Action::name(&crate::quick_open::GoToFile)
    });
    assert!(found, "cmd-p is bound to GoToFile");
}

/// BB-BUG-048: File > New Window, Cmd+Shift+N, as in VS Code.
#[test]
fn new_window_is_first_in_the_file_menu_and_on_cmd_shift_n() {
    let bound: bool = super::bindings().iter().any(|binding: &gpui::KeyBinding| {
        binding
            .keystrokes()
            .first()
            .is_some_and(|key: &gpui::KeybindingKeystroke| {
                key.inner().key == "n" && key.inner().modifiers == gpui::Modifiers::command_shift()
            })
            && binding.action().name() == gpui::Action::name(&super::NewWindow)
    });
    assert!(bound, "cmd-shift-n is bound to NewWindow");
    let menu: gpui::Menu = super::file_menu();
    assert!(matches!(
        menu.items.first(),
        Some(gpui::MenuItem::Action { name, .. }) if name.as_ref() == "New Window"
    ));
}
