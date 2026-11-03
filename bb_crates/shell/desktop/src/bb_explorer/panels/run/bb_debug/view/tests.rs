//! Exception breakpoints (BBEOD19) at the window: the Debug menu's actions
//! flip the ticks and say so; a paused panel names the exception above the
//! stack, and its ticks take a click.

use gpui::{Bounds, Modifiers, Pixels, TestAppContext, VisualTestContext};

use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
use crate::run::bb_debug::session::control::fake_paused;
use crate::run::bb_debug::session::Frame;
use crate::run::bb_debug::{BreakOnAllExceptions, BreakOnUserUnhandledExceptions, ExceptionBreaks};
use crate::ExplorerView;

fn ticks(rig: &Rig, vcx: &mut VisualTestContext) -> ExceptionBreaks {
    rig.view
        .read_with(vcx, |v: &ExplorerView, _: &gpui::App| v.debug.exceptions)
}

fn status(rig: &Rig, vcx: &mut VisualTestContext) -> Option<String> {
    rig.view
        .read_with(vcx, |v: &ExplorerView, _: &gpui::App| v.status.clone())
}

#[gpui::test]
fn the_debug_menu_actions_flip_the_ticks_and_say_so(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    assert_eq!(
        ticks(&rig, vcx),
        ExceptionBreaks::default(),
        "off until asked, as VS Code"
    );
    vcx.dispatch_action(BreakOnAllExceptions);
    vcx.run_until_parked();
    assert!(ticks(&rig, vcx).all);
    assert_eq!(
        status(&rig, vcx).as_deref(),
        Some("Break on all exceptions: on")
    );
    vcx.dispatch_action(BreakOnUserUnhandledExceptions);
    vcx.dispatch_action(BreakOnAllExceptions);
    vcx.run_until_parked();
    assert_eq!(
        ticks(&rig, vcx),
        ExceptionBreaks {
            all: false,
            user_unhandled: true,
        }
    );
    assert_eq!(
        status(&rig, vcx).as_deref(),
        Some("Break on all exceptions: off")
    );
}

#[gpui::test]
fn a_paused_panel_names_the_exception_and_its_ticks_take_a_click(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    let frame: Frame = Frame {
        id: 1,
        name: String::from("Program.Fail()"),
        path: Some(rig.file.clone()),
        line: 0,
    };
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            fake_paused(
                v,
                vec![frame],
                Some(String::from(
                    "System.InvalidOperationException: total was 14",
                )),
            );
            cx.notify();
        },
    );
    vcx.run_until_parked();
    assert!(
        vcx.debug_bounds("debug-exception").is_some(),
        "the exception row is drawn"
    );
    assert!(
        vcx.debug_bounds("debug-frame-0").is_some(),
        "above the stack"
    );
    let tick: Bounds<Pixels> = vcx
        .debug_bounds("exceptions-all")
        .expect("the All Exceptions tick");
    vcx.simulate_click(tick.center(), Modifiers::default());
    assert!(ticks(&rig, vcx).all, "a click on the tick turns it on");
    let described: String = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        super::notes::describe(v)
    });
    assert!(
        described.contains("exception=Some(\"System.InvalidOperationException: total was 14\")"),
        "{described}"
    );
}
