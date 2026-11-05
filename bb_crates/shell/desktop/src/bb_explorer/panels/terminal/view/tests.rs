//! The terminal panel in a window: keys typed there run in the shell.

use crate::intel::glue::window_tests::bb_rig::{rig, Rig};
use crate::ExplorerView;
use gpui::{TestAppContext, VisualTestContext};

fn rows(rig: &Rig, vcx: &mut VisualTestContext) -> Vec<String> {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.terminal
            .session
            .as_ref()
            .map(|s: &super::super::Session| s.screen().rows)
            .unwrap_or_default()
    })
}

/// Ctrl+backtick opens the shell; keys typed in the window run there and
/// the output is on the panel's screen.
#[gpui::test]
fn keys_typed_in_the_window_run_in_the_shell(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    vcx.dispatch_action(super::super::ToggleTerminal);
    vcx.run_until_parked();
    assert!(rig
        .view
        .read_with(vcx, |v: &ExplorerView, _: &gpui::App| v.terminal.open));
    vcx.simulate_keystrokes("e c h o space b b 4 2 o k enter");
    let mut seen: bool = false;
    for _ in 0..500 {
        vcx.run_until_parked();
        if rows(&rig, vcx)
            .iter()
            .any(|row: &String| row.contains("bb42ok"))
        {
            seen = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(seen, "{:?}", rows(&rig, vcx));
    vcx.dispatch_action(super::super::ToggleTerminal);
    assert!(!rig
        .view
        .read_with(vcx, |v: &ExplorerView, _: &gpui::App| v.terminal.open));
}

/// BB-BUG-053: a mouse drag over output selects it; Cmd+C copies exactly
/// that text; a typed key clears the selection.
#[gpui::test]
fn a_drag_selects_output_and_cmd_c_copies_it(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    vcx.dispatch_action(super::super::ToggleTerminal);
    vcx.run_until_parked();
    vcx.simulate_keystrokes("e c h o space b b _ c o p y _ m e");
    vcx.simulate_keystrokes("enter");
    let mut row: Option<usize> = None;
    for _ in 0..500 {
        vcx.run_until_parked();
        row = rows(&rig, vcx)
            .iter()
            .position(|r: &String| r.starts_with("bb_copy_me"));
        if row.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let row: f32 = f32::from(u16::try_from(row.expect("echoed")).unwrap());
    let grid: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds("terminal-grid").unwrap();
    let cell: f32 = f32::from(vcx.debug_bounds("terminal-cursor").unwrap().size.width);
    let count: f32 = f32::from(u16::try_from(rows(&rig, vcx).len()).unwrap());
    let line: f32 = f32::from(grid.size.height) / count;
    let (from, to): (gpui::Point<gpui::Pixels>, gpui::Point<gpui::Pixels>) = (
        at(grid, (cell, line), (0.1, row)),
        at(grid, (cell, line), (9.9, row)),
    );
    vcx.simulate_mouse_down(from, gpui::MouseButton::Left, gpui::Modifiers::default());
    vcx.simulate_mouse_move(
        to,
        Some(gpui::MouseButton::Left),
        gpui::Modifiers::default(),
    );
    vcx.simulate_mouse_up(to, gpui::MouseButton::Left, gpui::Modifiers::default());
    vcx.run_until_parked();
    assert!(vcx.debug_bounds("terminal-selected").is_some(), "drawn");
    vcx.simulate_keystrokes("cmd-c");
    let copied: Option<String> = vcx
        .read_from_clipboard()
        .and_then(|c: gpui::ClipboardItem| c.text());
    assert_eq!(copied.as_deref(), Some("bb_copy_me"));
    vcx.simulate_keystrokes("x");
    vcx.run_until_parked();
    assert!(
        vcx.debug_bounds("terminal-selected").is_none(),
        "a key clears it"
    );
    vcx.dispatch_action(super::super::ToggleTerminal);
}

/// The window point at `column` (in cells) on the middle of `row`.
fn at(
    grid: gpui::Bounds<gpui::Pixels>,
    (cell, line): (f32, f32),
    (column, row): (f32, f32),
) -> gpui::Point<gpui::Pixels> {
    grid.origin + gpui::point(gpui::px(column * cell), gpui::px((row + 0.5) * line))
}
