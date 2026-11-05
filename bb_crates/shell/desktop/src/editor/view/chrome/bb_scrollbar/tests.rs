//! BB-BUG-051 through the window: a bar shows only when the file overflows
//! its pane, and dragging its thumb scrolls.

use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext};

use super::thumb;
use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
use crate::ExplorerView;

fn settle(vcx: &mut VisualTestContext) {
    // The bars read the list's size from the frame before.
    vcx.run_until_parked();
    vcx.update(|window: &mut gpui::Window, _: &mut gpui::App| window.refresh());
    vcx.run_until_parked();
}

fn drag(vcx: &mut VisualTestContext, selector: &'static str, by: gpui::Point<gpui::Pixels>) {
    let from: gpui::Point<gpui::Pixels> = vcx.debug_bounds(selector).unwrap().center();
    vcx.simulate_mouse_down(from, MouseButton::Left, Modifiers::default());
    vcx.simulate_mouse_move(from + by, Some(MouseButton::Left), Modifiers::default());
    vcx.simulate_mouse_up(from + by, MouseButton::Left, Modifiers::default());
    vcx.run_until_parked();
}

fn scrolled_down(rig: &Rig, vcx: &mut VisualTestContext) -> f32 {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        -f32::from(v.tabs.scroll().0.borrow().base_handle.offset().y)
    })
}

#[gpui::test]
fn a_short_file_shows_no_bar(cx: &mut TestAppContext) {
    let (_rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    settle(vcx);
    assert!(vcx.debug_bounds("vbar-thumb-0").is_none());
    assert!(vcx.debug_bounds("hbar-thumb").is_none());
}

#[gpui::test]
fn a_tall_file_shows_a_bar_whose_thumb_drags_the_lines(cx: &mut TestAppContext) {
    let text: String = "// line\n".repeat(400);
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, &text);
    settle(vcx);
    assert!(
        vcx.debug_bounds("hbar-thumb").is_none(),
        "no line is too wide"
    );
    assert_eq!(scrolled_down(&rig, vcx), 0.0);
    drag(
        vcx,
        "vbar-thumb-0",
        gpui::point(gpui::px(0.0), gpui::px(100.0)),
    );
    let down: f32 = scrolled_down(&rig, vcx);
    assert!(down > 100.0, "the lines moved more than the thumb: {down}");
    let track: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds("vbar-0").unwrap();
    let near_end: gpui::Point<gpui::Pixels> =
        gpui::point(track.center().x, track.bottom() - gpui::px(2.0));
    vcx.simulate_click(near_end, Modifiers::default());
    vcx.run_until_parked();
    assert!(
        scrolled_down(&rig, vcx) > down,
        "a press on the track jumps there"
    );
}

#[gpui::test]
fn a_wide_line_shows_a_bottom_bar_whose_thumb_scrolls_across(cx: &mut TestAppContext) {
    let text: String = format!("class A\n// {}\n", "word ".repeat(300));
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, &text);
    settle(vcx);
    drag(
        vcx,
        "hbar-thumb",
        gpui::point(gpui::px(100.0), gpui::px(0.0)),
    );
    let across: f32 = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs.active_tab().unwrap().scroll_x
    });
    assert!(
        across > 100.0,
        "the text moved more than the thumb: {across}"
    );
}

#[test]
fn the_thumb_shows_the_share_in_sight_and_where() {
    assert_eq!(thumb(100.0, 0.0, 0.0), None, "all fits: no bar");
    let half: super::Thumb = thumb(100.0, 100.0, 50.0).unwrap();
    assert_eq!(
        (half.length, half.start),
        (50.0, 25.0),
        "half in sight, half way"
    );
    assert_eq!(half.per_px, 2.0);
    let huge: super::Thumb = thumb(100.0, 1_000_000.0, 1_000_000.0).unwrap();
    assert_eq!(huge.length, 24.0, "never shorter than a grip");
    assert_eq!(huge.start, 76.0, "at the end: the track's end");
}

/// BB-BUG-055: a cursor key that leaves the view scrolls just enough to show
/// the cursor's line again, both ways, as VS Code does.
#[gpui::test]
fn the_lines_follow_the_cursor_up_and_down(cx: &mut TestAppContext) {
    let text: String = "// line\n".repeat(400);
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, &text);
    settle(vcx);
    vcx.simulate_keystrokes(&"down ".repeat(120));
    settle(vcx);
    let down: f32 = scrolled_down(&rig, vcx);
    assert!(down > 0.0, "line 120 is below the first screen");
    vcx.simulate_keystrokes(&"up ".repeat(120));
    settle(vcx);
    assert_eq!(
        scrolled_down(&rig, vcx),
        0.0,
        "line 0 is back at the top (was {down})"
    );
    vcx.simulate_keystrokes("down");
    settle(vcx);
    assert_eq!(
        scrolled_down(&rig, vcx),
        0.0,
        "a move inside the view does not scroll"
    );
}
