//! BB-FEAT-079: the terminal splits into shells side by side, as VS Code.

use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext};

use crate::intel::glue::window_tests::bb_rig::{rig, Rig};
use crate::ExplorerView;

fn count(rig: &Rig, vcx: &mut VisualTestContext) -> (usize, usize) {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        (v.terminal.panes.len(), v.terminal.active)
    })
}

fn rows(rig: &Rig, vcx: &mut VisualTestContext, index: usize) -> Vec<String> {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.terminal
            .panes
            .get(index)
            .map(|p: &super::Pane| p.session.screen().rows)
            .unwrap_or_default()
    })
}

/// Wait until pane `index` shows `needle`.
fn shows(rig: &Rig, vcx: &mut VisualTestContext, index: usize, needle: &str) -> bool {
    for _ in 0..500 {
        vcx.run_until_parked();
        if rows(rig, vcx, index)
            .iter()
            .any(|row: &String| row.contains(needle))
        {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    false
}

fn click(vcx: &mut VisualTestContext, selector: &'static str) {
    let at: gpui::Point<gpui::Pixels> = vcx.debug_bounds(selector).expect(selector).center();
    vcx.simulate_click(at, Modifiers::default());
    vcx.run_until_parked();
}

/// Cmd+\ in the terminal adds a second shell to its right; each pane is
/// half as wide, the keys go to the new one, and the first keeps its own.
#[gpui::test]
fn cmd_backslash_splits_the_terminal_into_two_shells_side_by_side(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    vcx.update(|_: &mut gpui::Window, cx: &mut gpui::App| crate::launch::bind_keys(cx));
    vcx.dispatch_action(super::super::ToggleTerminal);
    vcx.run_until_parked();
    let whole: f32 = f32::from(vcx.debug_bounds("terminal-pane-0").unwrap().size.width);
    vcx.simulate_keystrokes("cmd-\\");
    vcx.run_until_parked();
    assert_eq!(count(&rig, vcx), (2, 1), "two panes, the new one active");
    let (left, right): (gpui::Bounds<gpui::Pixels>, gpui::Bounds<gpui::Pixels>) = (
        vcx.debug_bounds("terminal-pane-0").unwrap(),
        vcx.debug_bounds("terminal-pane-1").unwrap(),
    );
    assert!(right.origin.x > left.origin.x, "side by side");
    assert_eq!(right.origin.y, left.origin.y, "on one line");
    assert!(
        (f32::from(left.size.width) - whole / 2.0).abs() <= 2.0,
        "half each: {whole} -> {left:?}"
    );
    vcx.simulate_keystrokes("e c h o space b b _ r i g h t enter");
    assert!(shows(&rig, vcx, 1, "bb_right"), "{:?}", rows(&rig, vcx, 1));
    assert!(!rows(&rig, vcx, 0)
        .iter()
        .any(|r: &String| r.contains("bb_right")));
    let first: gpui::Point<gpui::Pixels> = vcx.debug_bounds("terminal-pane-0").unwrap().center();
    vcx.simulate_mouse_down(first, MouseButton::Left, Modifiers::default());
    vcx.simulate_mouse_up(first, MouseButton::Left, Modifiers::default());
    vcx.run_until_parked();
    assert_eq!(count(&rig, vcx), (2, 0), "a click takes the first pane");
    vcx.simulate_keystrokes("e c h o space b b _ l e f t enter");
    assert!(shows(&rig, vcx, 0, "bb_left"), "{:?}", rows(&rig, vcx, 0));
    vcx.dispatch_action(super::super::ToggleTerminal);
}

/// `+` adds a shell; a pane's ✕ ends only that one; one shell left shows
/// no chips.
#[gpui::test]
fn plus_adds_a_shell_and_a_panes_cross_ends_only_that_one(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    vcx.dispatch_action(super::super::ToggleTerminal);
    vcx.run_until_parked();
    assert!(
        vcx.debug_bounds("terminal-chip-0").is_none(),
        "one shell, no chips"
    );
    click(vcx, "terminal-split");
    click(vcx, "terminal-split");
    assert_eq!(count(&rig, vcx), (3, 2));
    click(vcx, "terminal-kill-1");
    assert_eq!(count(&rig, vcx), (2, 1), "the middle one gone");
    click(vcx, "terminal-kill-0");
    assert_eq!(count(&rig, vcx), (1, 0));
    assert!(vcx.debug_bounds("terminal-chip-0").is_none(), "one again");
    vcx.dispatch_action(super::super::ToggleTerminal);
}

/// Ctrl+Shift+backtick with the panel hidden opens it with one more shell.
#[gpui::test]
fn the_split_action_opens_a_hidden_terminal_with_one_more_shell(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    vcx.dispatch_action(super::super::ToggleTerminal);
    vcx.dispatch_action(super::super::ToggleTerminal);
    vcx.run_until_parked();
    vcx.dispatch_action(super::SplitTerminal);
    vcx.run_until_parked();
    assert_eq!(count(&rig, vcx), (2, 1));
    assert!(vcx.debug_bounds("terminal-pane-1").is_some(), "shown");
    vcx.dispatch_action(super::super::ToggleTerminal);
}

/// `exit` in one shell closes its pane; the other stays. `exit` in the last
/// one hides the panel, and Ctrl+backtick then starts a fresh shell.
#[gpui::test]
fn exit_in_a_shell_closes_only_its_pane(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    vcx.dispatch_action(super::super::ToggleTerminal);
    vcx.dispatch_action(super::SplitTerminal);
    vcx.run_until_parked();
    exit_to(&rig, vcx, 1);
    assert!(
        vcx.debug_bounds("terminal-pane-0").is_some(),
        "the other stays"
    );
    exit_to(&rig, vcx, 0);
    vcx.run_until_parked();
    assert!(
        vcx.debug_bounds("terminal").is_none(),
        "the last one hides it"
    );
    vcx.dispatch_action(super::super::ToggleTerminal);
    vcx.run_until_parked();
    assert_eq!(count(&rig, vcx), (1, 0), "a fresh shell");
    vcx.dispatch_action(super::super::ToggleTerminal);
}

/// Type `exit` in the active shell and wait until `left` panes remain.
fn exit_to(rig: &Rig, vcx: &mut VisualTestContext, left: usize) {
    vcx.simulate_keystrokes("e x i t enter");
    let mut closed: bool = false;
    for _ in 0..500 {
        vcx.update(|window: &mut gpui::Window, cx: &mut gpui::App| {
            rig.view.update(
                cx,
                |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
                    super::reap(v, window, cx);
                },
            );
        });
        if count(rig, vcx).0 == left {
            closed = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(closed, "{:?}", count(rig, vcx));
}
