//! Back / Forward at the window (BB-FEAT-090, BB-BUG-067): a debugger stop
//! in another file is a place Back returns to; the title bar's ← and → are
//! grey until there is a place to go, and the keys work from any focus.

use std::path::PathBuf;

use gpui::{Bounds, Modifiers, Pixels, TestAppContext, VisualTestContext};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
use crate::run::bb_debug::session::control::fake_paused;
use crate::run::bb_debug::session::{Command, Frame, Note};
use crate::ExplorerView;

fn active(rig: &Rig, vcx: &mut VisualTestContext) -> Option<PathBuf> {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs
            .active_tab()
            .map(|tab: &crate::editor::Tab| tab.path.clone())
    })
}

/// Paused in A.cs, then a step into B.cs.
fn step_into(rig: &Rig, vcx: &mut VisualTestContext, other: &std::path::Path) {
    let at: fn(&std::path::Path, i64) -> Frame = |path: &std::path::Path, id: i64| Frame {
        id,
        name: String::from("M()"),
        path: Some(path.to_path_buf()),
        line: 0,
    };
    let first: Frame = at(&rig.file, 1);
    let _sent: UnboundedReceiver<Command> = rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            let sent: UnboundedReceiver<Command> = fake_paused(v, vec![first], None);
            cx.notify();
            sent
        },
    );
    let stop: Note = Note::Stopped {
        reason: String::from("step"),
        exception: None,
        frames: vec![at(other, 2), at(&rig.file, 1)],
        vars: Vec::new(),
    };
    vcx.update(|window: &mut gpui::Window, cx: &mut gpui::App| {
        rig.view.update(
            cx,
            |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
                crate::run::bb_debug::view::notes::apply(v, stop, window, cx);
                cx.notify();
            },
        );
    });
    vcx.run_until_parked();
}

#[gpui::test]
fn back_returns_from_a_step_into_another_file_by_the_title_bar_or_the_key(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    let other: PathBuf = rig.root.join("B.cs");
    std::fs::write(&other, "class B {}\n").unwrap();
    assert!(
        vcx.debug_bounds("nav-back-off").is_some(),
        "grey: nowhere to go"
    );
    assert!(vcx.debug_bounds("nav-forward-off").is_some());
    step_into(&rig, vcx, &other);
    assert_eq!(
        active(&rig, vcx),
        Some(other.clone()),
        "the stop opened B.cs"
    );
    let back: Bounds<Pixels> = vcx.debug_bounds("nav-back").expect("← is live");
    vcx.simulate_click(back.center(), Modifiers::default());
    vcx.run_until_parked();
    assert_eq!(
        active(&rig, vcx),
        Some(rig.file.clone()),
        "← went back to A.cs"
    );
    assert!(vcx.debug_bounds("nav-forward").is_some(), "→ is live now");
    vcx.dispatch_action(crate::editor::GoForward);
    vcx.run_until_parked();
    assert_eq!(active(&rig, vcx), Some(other), "Cmd+] went forward again");
}

/// BB-BUG-070: a double click on the bar zooms once and never starts the
/// system drag, which zoomed a second time; a press drags only on a move.
#[test]
fn a_double_click_zooms_once_and_only_a_moved_press_drags() {
    use super::TitleDrag;
    let mut drag: TitleDrag = TitleDrag::default();
    drag.press(1);
    assert!(!drag.release(1), "one click is not a zoom");
    assert!(!drag.moved(true), "a released press no longer drags");
    drag.press(2);
    assert!(
        !drag.moved(true),
        "the second press of a double click never drags"
    );
    assert!(drag.release(2), "the double click zooms on release");
    drag.press(1);
    assert!(drag.moved(true), "a moved press drags the window");
    assert!(!drag.moved(true), "the drag starts once per press");
    drag.press(1);
    assert!(
        !drag.moved(false),
        "a move after a release outside the bar does not drag"
    );
    assert!(!drag.moved(true), "and the press stays disarmed");
    drag.press(3);
    assert!(!drag.release(3), "a triple click does not zoom again");
}
