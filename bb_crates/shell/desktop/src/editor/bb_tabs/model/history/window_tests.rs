//! Back / Forward in the window: an F12 jump, a closed tab reopened at its
//! place, a deleted file skipped with the arrow off (BB-BUG-080).

use std::path::{Path, PathBuf};

use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext};

use super::{GoBack, GoForward, Place};
use crate::intel::bb_worker::protocol::{Answer, Ticket};
use crate::intel::glue::window_tests::bb_rig::{place, rig, rig_with, Rig};
use crate::ExplorerView;

const LINES: &str = "line0\nline1 here\nline2\nline3\n";

fn active(rig: &Rig, vcx: &mut VisualTestContext) -> Option<PathBuf> {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs
            .active_tab()
            .map(|tab: &crate::editor::Tab| tab.path.clone())
    })
}

/// The active file and its cursor line.
fn here(rig: &Rig, vcx: &mut VisualTestContext) -> Option<(PathBuf, u32)> {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs.here().map(|p: Place| (p.path, p.at.line))
    })
}

fn status(rig: &Rig, vcx: &mut VisualTestContext) -> String {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.status.clone().unwrap_or_default()
    })
}

/// F12 answered with `to`'s line `line`.
fn f12(rig: &mut Rig, vcx: &mut VisualTestContext, to: &Path, line: u32) {
    vcx.simulate_keystrokes("f12");
    let (ticket, _): (Ticket, _) = rig.last_ask();
    rig.reply(vcx, ticket, Answer::Definition(vec![place(to, line)]));
    vcx.run_until_parked();
}

fn press(vcx: &mut VisualTestContext, selector: &'static str) {
    let at: gpui::Point<gpui::Pixels> = vcx.debug_bounds(selector).unwrap().center();
    vcx.simulate_mouse_down(at, MouseButton::Left, Modifiers::default());
    vcx.simulate_mouse_up(at, MouseButton::Left, Modifiers::default());
    vcx.run_until_parked();
}

/// Close `path`'s tab by its ✕.
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

/// The user leaves the window, deletes `path` elsewhere, and comes back.
fn delete_in_another_app(vcx: &mut VisualTestContext, path: &Path) {
    vcx.deactivate_window();
    std::fs::remove_file(path).unwrap();
    vcx.update(|window: &mut gpui::Window, _: &mut gpui::App| window.activate_window());
    vcx.run_until_parked();
}

fn keys(vcx: &mut VisualTestContext, keystrokes: &str) {
    vcx.simulate_keystrokes(keystrokes);
    vcx.run_until_parked();
}

/// F12 into another file, then Back returns to the first file and line,
/// and Forward goes to the target again (the real actions on the editor).
#[gpui::test]
fn back_and_forward_retrace_an_f12_jump_in_the_window(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    let other: PathBuf = rig.root.join("Other.cs");
    std::fs::write(&other, "line0\nclass Other {}\n").unwrap();
    let file: PathBuf = rig.file.clone();
    f12(&mut rig, vcx, &other, 1);
    assert_eq!(active(&rig, vcx), Some(other.clone()));
    vcx.dispatch_action(GoBack);
    assert_eq!(
        active(&rig, vcx),
        Some(file),
        "Back returns to the first file"
    );
    vcx.dispatch_action(GoForward);
    assert_eq!(
        active(&rig, vcx),
        Some(other),
        "Forward goes to the target again"
    );
}

#[gpui::test]
fn back_by_mouse_reopens_a_closed_file_at_its_line_and_is_off_once_the_file_is_deleted(
    cx: &mut TestAppContext,
) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, LINES);
    vcx.update(|_: &mut gpui::Window, cx: &mut gpui::App| crate::launch::bind_keys(cx));
    let (a, b): (PathBuf, PathBuf) = (rig.file.clone(), rig.root.join("B.cs"));
    std::fs::write(&b, LINES).unwrap();
    rig.cursor_after(vcx, "line1 ");
    f12(&mut rig, vcx, &b, 3);
    close_by_mouse(&rig, vcx, &a);
    assert_eq!(active(&rig, vcx), Some(b.clone()), "A.cs is closed");
    press(vcx, "nav-back");
    assert_eq!(
        here(&rig, vcx),
        Some((a.clone(), 1)),
        "← reopened A.cs at line 1"
    );
    keys(vcx, "ctrl-shift--");
    assert_eq!(
        here(&rig, vcx),
        Some((b.clone(), 3)),
        "Ctrl+Shift+- went forward"
    );
    close_by_mouse(&rig, vcx, &a);
    assert!(
        vcx.debug_bounds("nav-back").is_some(),
        "← is live: A.cs is on disk"
    );
    delete_in_another_app(vcx, &a);
    assert!(vcx.debug_bounds("nav-back").is_none(), "← is not live");
    assert!(
        vcx.debug_bounds("nav-back-off").is_some(),
        "← is grey: A.cs is deleted"
    );
    keys(vcx, "cmd-[");
    assert_eq!(
        here(&rig, vcx),
        Some((b, 3)),
        "nothing to reach: B.cs stays"
    );
    let said: String = status(&rig, vcx);
    assert!(
        said.contains("A.cs") && said.contains("no longer exists"),
        "{said}"
    );
}

#[gpui::test]
fn with_no_tab_left_ctrl_minus_reopens_the_closed_file_at_its_line(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, LINES);
    vcx.update(|_: &mut gpui::Window, cx: &mut gpui::App| crate::launch::bind_keys(cx));
    let a: PathBuf = rig.file.clone();
    rig.cursor_after(vcx, "line1 ");
    f12(&mut rig, vcx, &a, 3);
    assert_eq!(here(&rig, vcx), Some((a.clone(), 3)), "a jump inside A.cs");
    close_by_mouse(&rig, vcx, &a);
    assert_eq!(active(&rig, vcx), None, "no tab is open");
    assert!(vcx.debug_bounds("nav-back").is_some(), "← is live");
    keys(vcx, "ctrl--");
    assert_eq!(
        here(&rig, vcx),
        Some((a, 1)),
        "Ctrl+- reopened A.cs at line 1"
    );
    assert!(
        vcx.debug_bounds("nav-back-off").is_some(),
        "nothing further back"
    );
}
