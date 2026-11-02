//! BBEOD17 part 1: F9, a click on a line number, and edits above a breakpoint.

use gpui::{Bounds, Modifiers, Pixels, TestAppContext, VisualTestContext};

use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
use crate::ExplorerView;

const TEXT: &str = "class A\n{\n    void M() { }\n}\n";

fn has(rig: &Rig, vcx: &mut VisualTestContext, line: usize) -> bool {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.debug.breakpoints.has(&rig.file, line)
    })
}

fn put_cursor(rig: &Rig, vcx: &mut VisualTestContext, line: usize, column: usize) {
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            let doc: &mut bb_text::Document = v.tabs.active_mut().unwrap();
            let start: usize = doc.line_start(line).unwrap().raw();
            doc.set_cursor(bb_text::CharIdx::new(start + column))
                .unwrap();
            cx.notify();
        },
    );
    vcx.run_until_parked();
}

#[gpui::test]
fn f9_toggles_a_breakpoint_on_the_cursor_line_and_the_gutter_shows_it(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, TEXT);
    put_cursor(&rig, vcx, 2, 4);
    vcx.dispatch_action(super::ToggleBreakpoint);
    vcx.run_until_parked();
    assert!(has(&rig, vcx, 2));
    assert!(
        vcx.debug_bounds("breakpoint-2").is_some(),
        "the red tag is drawn"
    );
    assert!(vcx.debug_bounds("breakpoint-1").is_none());
    vcx.dispatch_action(super::ToggleBreakpoint);
    vcx.run_until_parked();
    assert!(!has(&rig, vcx, 2));
    assert!(vcx.debug_bounds("breakpoint-2").is_none());
}

#[gpui::test]
fn a_click_on_a_line_number_toggles_it_and_the_cursor_stays(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, TEXT);
    put_cursor(&rig, vcx, 0, 2);
    let number: Bounds<Pixels> = vcx.debug_bounds("line-number-3").unwrap();
    vcx.simulate_click(number.center(), Modifiers::default());
    assert!(has(&rig, vcx, 3));
    let cursor: usize = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs.active().unwrap().cursor().raw()
    });
    assert_eq!(cursor, 2, "a gutter click never moves the cursor");
}

#[gpui::test]
fn enter_above_a_breakpoint_carries_it_down_with_its_code(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, TEXT);
    put_cursor(&rig, vcx, 2, 4);
    vcx.dispatch_action(super::ToggleBreakpoint);
    put_cursor(&rig, vcx, 1, 1);
    vcx.simulate_keystrokes("enter");
    assert!(has(&rig, vcx, 3), "the breakpoint moved with `void M`");
    assert!(!has(&rig, vcx, 2));
    vcx.simulate_keystrokes("backspace");
    assert!(has(&rig, vcx, 2), "and back up when the line goes");
}

/// The Debuggee fixture and a solution for it under the rig's root.
fn debuggee(rig: &Rig) -> std::path::PathBuf {
    let source: std::path::PathBuf =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/Debuggee");
    let dir: std::path::PathBuf = rig.root.join("Debuggee");
    std::fs::create_dir_all(&dir).unwrap();
    for name in ["Debuggee.csproj", "_Program.cs"] {
        std::fs::copy(source.join(name), dir.join(name)).unwrap();
    }
    std::fs::write(
        rig.root.join("Debuggee.slnx"),
        "<Solution>\n  <Project Path=\"Debuggee/Debuggee.csproj\" />\n</Solution>\n",
    )
    .unwrap();
    dir.join("_Program.cs")
}

/// The Debuggee open in a tab with a breakpoint on `int result = n * n;`.
fn open_with_breakpoint(rig: &Rig, vcx: &mut VisualTestContext) -> std::path::PathBuf {
    let program: std::path::PathBuf = debuggee(rig);
    rig.view.update_in(
        vcx,
        |v: &mut ExplorerView, w: &mut gpui::Window, cx: &mut gpui::Context<ExplorerView>| {
            v.open_file(&program, w, cx);
        },
    );
    vcx.run_until_parked();
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, _: &mut gpui::Context<ExplorerView>| {
            super::toggle_line(v, &program, 17);
        },
    );
    program
}

/// Real time passes: the session runs on its own thread.
fn wait_until(
    rig: &Rig,
    vcx: &mut VisualTestContext,
    what: &str,
    done: fn(&ExplorerView, &std::path::Path) -> bool,
    file: &std::path::Path,
) {
    for _ in 0..12_000 {
        rig.view.update_in(
            vcx,
            |v: &mut ExplorerView, w: &mut gpui::Window, cx: &mut gpui::Context<ExplorerView>| {
                super::session::control::poll(v, w, cx);
            },
        );
        vcx.run_until_parked();
        if rig
            .view
            .read_with(vcx, |v: &ExplorerView, _: &gpui::App| done(v, file))
        {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let status: Option<String> = rig
        .view
        .read_with(vcx, |v: &ExplorerView, _: &gpui::App| v.status.clone());
    panic!("timed out waiting for {what}; status {status:?}");
}

/// BBEOD17 part 2, on the real netcoredbg: F5 builds and stops on the
/// breakpoint, the line is highlighted, the call stack lists `Main`; a click
/// on that frame opens its call site; F10 steps; Shift+F5 ends.
#[gpui::test]
fn f5_stops_shows_the_stack_opens_a_call_site_steps_and_stops(cx: &mut TestAppContext) {
    bb_dap::locate().expect("netcoredbg must be installed for this gate");
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    let program: std::path::PathBuf = open_with_breakpoint(&rig, vcx);
    vcx.dispatch_action(super::StartOrContinue);
    wait_until(
        &rig,
        vcx,
        "the first stop",
        |v: &ExplorerView, f: &std::path::Path| super::stopped_line(v, f) == Some(17),
        &program,
    );
    assert!(
        vcx.debug_bounds("stopped-line-17").is_some(),
        "the paused line is highlighted"
    );
    let caller: Bounds<Pixels> = vcx
        .debug_bounds("debug-frame-1")
        .expect("Main in the call stack");
    vcx.simulate_click(caller.center(), Modifiers::default());
    wait_until(
        &rig,
        vcx,
        "the call site",
        |v: &ExplorerView, f: &std::path::Path| super::stopped_line(v, f) == Some(9),
        &program,
    );
    let cursor: usize = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs.active().unwrap().cursor_line()
    });
    assert_eq!(cursor, 9, "the caller's line, `total += Square(i);`");
    vcx.dispatch_action(super::StepOver);
    wait_until(
        &rig,
        vcx,
        "the step",
        |v: &ExplorerView, f: &std::path::Path| super::stopped_line(v, f) == Some(18),
        &program,
    );
    vcx.dispatch_action(super::StopDebugging);
    wait_until(
        &rig,
        vcx,
        "the end",
        |v: &ExplorerView, _: &std::path::Path| v.debug.live.is_none(),
        &program,
    );
}

/// A debug session builds the project (bin/, obj/): `--no-write` refuses
/// it, as it refuses Build.
#[gpui::test]
fn f5_in_no_write_mode_refuses_and_starts_nothing(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, true, true, TEXT);
    vcx.dispatch_action(super::StartOrContinue);
    vcx.run_until_parked();
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        assert!(v.debug.live.is_none());
        assert!(v
            .status
            .as_deref()
            .unwrap()
            .starts_with("Debug refused: --no-write"));
    });
}
