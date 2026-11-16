//! Exception breakpoints (BBEOD19) at the window: the Debug menu's actions
//! flip the ticks and say so; a paused panel names the exception above the
//! stack, and its ticks take a click. Set Next Statement (BBEOD05) only
//! where the adapter can.

use gpui::{Bounds, Modifiers, Pixels, TestAppContext, VisualTestContext};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
use crate::run::bb_debug::session::control::fake_paused;
use crate::run::bb_debug::session::{Command, Frame};
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

/// Pause in a fake session; the commands the window then sends.
fn paused(rig: &Rig, vcx: &mut VisualTestContext, line: usize) -> UnboundedReceiver<Command> {
    let frame: Frame = Frame {
        id: 1,
        name: String::from("A.M()"),
        path: Some(rig.file.clone()),
        line,
    };
    let sent: UnboundedReceiver<Command> = rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            cx.notify();
            fake_paused(v, vec![frame], None)
        },
    );
    vcx.run_until_parked();
    sent
}

#[gpui::test]
fn ctrl_shift_f10_sets_the_next_statement_only_where_the_adapter_can(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) =
        rig_with(cx, false, true, "class A\n{\n    void M() { }\n}\n");
    // The rig installs no app bindings (BB-UIX-010): install the app's own.
    vcx.update(|_: &mut gpui::Window, cx: &mut gpui::App| crate::launch::bind_keys(cx));
    let mut sent: UnboundedReceiver<Command> = paused(&rig, vcx, 2);
    vcx.simulate_keystrokes("ctrl-shift-f10");
    assert!(
        sent.try_recv().is_err(),
        "stock netcoredbg: no Set Next, nothing sent"
    );
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            if let Some(live) = v.debug.live.as_mut() {
                live.can_set_next = true;
            }
            let doc: &mut bb_text::Document = v.tabs.active_mut().unwrap();
            doc.set_cursor(bb_text::CharIdx::new(10)).unwrap();
            cx.notify();
        },
    );
    vcx.run_until_parked();
    vcx.simulate_keystrokes("ctrl-shift-f10");
    match sent.try_recv() {
        Ok(Command::SetNext(path, line)) => {
            assert_eq!(path, rig.file);
            assert_eq!(line, 2, "the cursor line, 0-based");
        }
        other => panic!("expected Set Next on the cursor line, got {other:?}"),
    }
}

/// Click `selector`; the command it sent, as text.
fn next(
    vcx: &mut VisualTestContext,
    sent: &mut UnboundedReceiver<Command>,
    selector: &'static str,
) -> String {
    let at: Bounds<Pixels> = vcx.debug_bounds(selector).expect(selector);
    vcx.simulate_click(at.center(), Modifiers::default());
    vcx.run_until_parked();
    format!("{:?}", sent.try_recv())
}

/// BB-FEAT-082, as Rider's debug toolbar: while paused, Resume, Step Over,
/// Into and Out send what F5, F10, F11 and Shift+F11 send; Stop ends it.
#[gpui::test]
fn the_debug_toolbar_resumes_steps_and_stops(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) =
        rig_with(cx, false, true, "class A\n{\n    void M() { }\n}\n");
    let mut sent: UnboundedReceiver<Command> = paused(&rig, vcx, 2);
    assert_eq!(next(vcx, &mut sent, "debug-resume"), "Ok(Continue)");
    assert_eq!(next(vcx, &mut sent, "debug-step-over"), "Ok(Step(Over))");
    assert_eq!(next(vcx, &mut sent, "debug-step-into"), "Ok(Step(Into))");
    assert_eq!(next(vcx, &mut sent, "debug-step-out"), "Ok(Step(Out))");
    assert_eq!(next(vcx, &mut sent, "debug-stop"), "Ok(Stop)");
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            if let Some(live) = v.debug.live.as_mut() {
                live.phase = crate::run::bb_debug::session::control::Phase::Running;
            }
            cx.notify();
        },
    );
    vcx.run_until_parked();
    assert_eq!(
        next(vcx, &mut sent, "debug-step-over"),
        "Err(Empty)",
        "running: the steps are greyed and do nothing"
    );
}

/// Drag the arrow on line 2 to line 0.
fn drag_arrow(vcx: &mut VisualTestContext) {
    let from: gpui::Point<Pixels> = vcx.debug_bounds("stopped-line-2").unwrap().center();
    let to: gpui::Point<Pixels> = vcx.debug_bounds("editor-line-0").unwrap().center();
    vcx.simulate_mouse_down(from, gpui::MouseButton::Left, Modifiers::default());
    vcx.simulate_mouse_move(to, Some(gpui::MouseButton::Left), Modifiers::default());
    vcx.simulate_mouse_up(to, gpui::MouseButton::Left, Modifiers::default());
    vcx.run_until_parked();
}

/// BB-FEAT-086, as Rider: the paused line's arrow drags to another line,
/// which then runs next; a debugger that cannot move it says so.
#[gpui::test]
fn dragging_the_arrow_to_a_line_sets_the_next_statement(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) =
        rig_with(cx, false, true, "class A\n{\n    void M() { }\n}\n");
    let mut sent: UnboundedReceiver<Command> = paused(&rig, vcx, 2);
    drag_arrow(vcx);
    assert!(sent.try_recv().is_err(), "stock netcoredbg: nothing sent");
    assert!(status(&rig, vcx).is_some_and(|s: String| s.starts_with("Set Next Statement")));
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            if let Some(live) = v.debug.live.as_mut() {
                live.can_set_next = true;
            }
            cx.notify();
        },
    );
    vcx.run_until_parked();
    drag_arrow(vcx);
    match sent.try_recv() {
        Ok(Command::SetNext(path, line)) => {
            assert_eq!(path, rig.file);
            assert_eq!(line, 0, "the line the arrow was dropped on");
        }
        other => panic!("expected Set Next on line 0, got {other:?}"),
    }
}
