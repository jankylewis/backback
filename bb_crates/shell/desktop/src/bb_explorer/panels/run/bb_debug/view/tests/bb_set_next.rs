//! BB-BUG-071: Set Next Statement inside an async spec, at the window. The
//! debugger offers a target only where a statement starts, so the arrow
//! dropped on a later line of a multi-line `await` runs that statement; a
//! refusal is said in the status bar and in the Debug output.

use gpui::{Modifiers, Pixels, TestAppContext, VisualTestContext};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
use crate::run::bb_debug::session::{Command, Note};
use crate::ExplorerView;

const SPEC: &str = "class A\n{\n    async Task M()\n    {\n        string t = await Task.FromResult(\"a\")\n            .ContinueWith(x => x.Result);\n        int n = 1;\n    }\n}\n";

/// Drag the arrow on line `from` to line `to`.
fn drag(vcx: &mut VisualTestContext, from: usize, to: usize) {
    let start: gpui::Point<Pixels> = vcx
        .debug_bounds(format!("stopped-line-{from}").leak())
        .unwrap()
        .center();
    let end: gpui::Point<Pixels> = vcx
        .debug_bounds(format!("editor-line-{to}").leak())
        .unwrap()
        .center();
    vcx.simulate_mouse_down(start, gpui::MouseButton::Left, Modifiers::default());
    vcx.simulate_mouse_move(end, Some(gpui::MouseButton::Left), Modifiers::default());
    vcx.simulate_mouse_up(end, gpui::MouseButton::Left, Modifiers::default());
    vcx.run_until_parked();
}

#[gpui::test]
fn the_arrow_dropped_inside_a_multi_line_await_runs_that_statement(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, SPEC);
    let mut sent: UnboundedReceiver<Command> = super::paused(&rig, vcx, 6);
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
    drag(vcx, 6, 5);
    match sent.try_recv() {
        Ok(Command::SetNext(path, line)) => {
            assert_eq!(path, rig.file);
            assert_eq!(line, 4, "the `await` chain's first line, not its 2nd");
        }
        other => panic!("expected Set Next on line 4, got {other:?}"),
    }
    let why: &str = "Set Next Statement refused: line 2 is not a target in the paused method";
    vcx.update(|window: &mut gpui::Window, app: &mut gpui::App| {
        rig.view.update(
            app,
            |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
                let note: Note = Note::Refused(why.to_owned());
                crate::run::bb_debug::view::notes::apply(v, note, window, cx);
            },
        );
    });
    let (status, output): (Option<String>, String) =
        rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
            let output: String = v
                .debug
                .live
                .as_ref()
                .map(|l: &crate::run::bb_debug::session::control::Live| l.output.clone())
                .unwrap_or_default();
            (v.status.clone(), output)
        });
    assert_eq!(status.as_deref(), Some(why), "the status bar says it");
    assert!(
        output.contains(why),
        "the Debug output keeps it: {output:?}"
    );
}
