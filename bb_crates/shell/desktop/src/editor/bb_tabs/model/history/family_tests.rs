//! BB-BUG-080's family: every other kept target that points at a tab (a
//! Places / Find in Files / Problems row, a test row's "go to") reopens a
//! closed tab at its line, and a deleted file says why and leaves no
//! pending jump behind.

use std::path::{Path, PathBuf};

use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext};

use super::Place;
use crate::intel::glue::window_tests::bb_rig::{place, rig_with, Rig};
use crate::ExplorerView;

const LINES: &str = "line0\nline1\nline2\nline3\n";

fn here(rig: &Rig, vcx: &mut VisualTestContext) -> Option<(PathBuf, u32)> {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs.here().map(|p: Place| (p.path, p.at.line))
    })
}

fn press(vcx: &mut VisualTestContext, selector: &'static str) {
    let at: gpui::Point<gpui::Pixels> = vcx.debug_bounds(selector).unwrap().center();
    vcx.simulate_mouse_down(at, MouseButton::Left, Modifiers::default());
    vcx.simulate_mouse_up(at, MouseButton::Left, Modifiers::default());
    vcx.run_until_parked();
}

fn close_by_mouse(rig: &Rig, vcx: &mut VisualTestContext, path: &Path) {
    let index: usize = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs
            .items()
            .iter()
            .position(|tab: &crate::editor::Tab| tab.path == path)
            .unwrap()
    });
    press(vcx, format!("close-{index}").leak());
}

/// The Places panel (references, Find in Files) with one row: `path`'s `line`.
fn show_place(rig: &Rig, vcx: &mut VisualTestContext, path: &Path, line: u32) {
    let row: crate::intel::bb_worker::protocol::Place = place(path, line);
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            v.intel.references = Some(crate::intel::model::popups::PlacesPanel {
                title: String::from("1 result"),
                places: vec![row],
                pick: crate::intel::model::popups::Pick::default(),
            });
            cx.notify();
        },
    );
    vcx.run_until_parked();
}

fn left_behind(rig: &Rig, vcx: &mut VisualTestContext) -> (bool, String) {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        (v.intel.jump.is_some(), v.status.clone().unwrap_or_default())
    })
}

fn open(rig: &Rig, vcx: &mut VisualTestContext, path: &Path) {
    let file: PathBuf = path.to_path_buf();
    rig.view.update_in(
        vcx,
        |v: &mut ExplorerView, window: &mut gpui::Window, cx: &mut gpui::Context<ExplorerView>| {
            v.open_file(&file, window, cx);
        },
    );
    vcx.run_until_parked();
}

#[gpui::test]
fn a_places_row_reopens_its_closed_file_and_a_deleted_one_says_why_and_leaves_no_jump(
    cx: &mut TestAppContext,
) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, LINES);
    let b: PathBuf = rig.root.join("B.cs");
    std::fs::write(&b, LINES).unwrap();
    show_place(&rig, vcx, &b, 2);
    press(vcx, "place-0");
    assert_eq!(here(&rig, vcx), Some((b.clone(), 2)), "the row opened B.cs");
    close_by_mouse(&rig, vcx, &b);
    show_place(&rig, vcx, &b, 2);
    press(vcx, "place-0");
    assert_eq!(
        here(&rig, vcx),
        Some((b.clone(), 2)),
        "the row reopened B.cs"
    );
    close_by_mouse(&rig, vcx, &b);
    std::fs::remove_file(&b).unwrap();
    show_place(&rig, vcx, &b, 2);
    press(vcx, "place-0");
    let (pending, said): (bool, String) = left_behind(&rig, vcx);
    assert!(said.contains("B.cs"), "the status says why: {said}");
    assert!(!pending, "no jump waits for a file that failed to open");
    std::fs::write(&b, LINES).unwrap();
    open(&rig, vcx, &b);
    assert_eq!(
        here(&rig, vcx),
        Some((b, 0)),
        "a later plain open is not moved by the dead jump"
    );
}

#[gpui::test]
fn a_test_rows_go_to_reopens_its_closed_file_and_a_deleted_one_says_why(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, LINES);
    let b: PathBuf = rig.root.join("B.cs");
    std::fs::write(&b, LINES).unwrap();
    let go: &dyn Fn(&mut VisualTestContext) = &|vcx: &mut VisualTestContext| {
        let target: PathBuf = b.clone();
        rig.view.update_in(
            vcx,
            |v: &mut ExplorerView, w: &mut gpui::Window, cx: &mut gpui::Context<ExplorerView>| {
                crate::tests_panel::navigation::jump_to(v, (&target, 3), w, cx);
            },
        );
        vcx.run_until_parked();
    };
    go(vcx);
    close_by_mouse(&rig, vcx, &b);
    go(vcx);
    assert_eq!(here(&rig, vcx), Some((b.clone(), 3)), "go to reopened B.cs");
    close_by_mouse(&rig, vcx, &b);
    std::fs::remove_file(&b).unwrap();
    go(vcx);
    let said: String = left_behind(&rig, vcx).1;
    assert!(said.contains("B.cs"), "the status says why: {said}");
    assert!(
        said.ends_with("the file no longer exists"),
        "a deleted local file is not 'not on this machine': {said}"
    );
}

fn chmod(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
}

/// A file the load cannot read (its path resolves, its text does not): the
/// row and the test "go to" both say why and leave no jump waiting.
#[gpui::test]
fn an_unreadable_file_says_why_and_leaves_no_waiting_jump_from_a_row_or_a_test(
    cx: &mut TestAppContext,
) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, LINES);
    let b: PathBuf = rig.root.join("B.cs");
    std::fs::write(&b, LINES).unwrap();
    chmod(&b, 0o000);
    show_place(&rig, vcx, &b, 2);
    press(vcx, "place-0");
    let target: PathBuf = b.clone();
    rig.view.update_in(
        vcx,
        |v: &mut ExplorerView, w: &mut gpui::Window, cx: &mut gpui::Context<ExplorerView>| {
            crate::tests_panel::navigation::jump_to(v, (&target, 3), w, cx);
        },
    );
    vcx.run_until_parked();
    let (pending, said): (bool, String) = left_behind(&rig, vcx);
    let waiting: bool = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tests.pending_jump.is_some()
    });
    chmod(&b, 0o644);
    assert!(said.starts_with("Cannot open B.cs"), "{said}");
    assert!(!pending && !waiting, "no jump waits: {pending} {waiting}");
    open(&rig, vcx, &b);
    assert_eq!(
        here(&rig, vcx),
        Some((b, 0)),
        "a later open stays at the top"
    );
}

/// BB-BUG-082: a breakpoint in a file deleted since it was set is not sent at
/// Debug start; the status names the file and the store forgets it.
#[gpui::test]
fn a_debug_start_drops_a_deleted_files_breakpoints_and_says_so(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, LINES);
    let b: PathBuf = rig.root.join("B.cs");
    std::fs::write(&b, LINES).unwrap();
    let a: PathBuf = rig.root.join("A.cs");
    std::fs::write(&a, LINES).unwrap();
    let sent: Vec<(PathBuf, Vec<usize>)> = rig.view.update(
        vcx,
        |v: &mut ExplorerView, _: &mut gpui::Context<ExplorerView>| {
            v.debug.breakpoints.toggle(&a, 1);
            v.debug.breakpoints.toggle(&b, 2);
            std::fs::remove_file(&b).unwrap();
            crate::run::bb_debug::for_start(v)
        },
    );
    assert_eq!(sent, vec![(a.clone(), vec![1])], "only A.cs is sent");
    let said: String = left_behind(&rig, vcx).1;
    assert_eq!(
        said,
        "Debug: breakpoints in B.cs dropped: the file no longer exists"
    );
    let kept: bool = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.debug.breakpoints.has(&b, 2)
    });
    assert!(!kept, "the dropped breakpoint is not sent again");
}
