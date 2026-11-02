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
