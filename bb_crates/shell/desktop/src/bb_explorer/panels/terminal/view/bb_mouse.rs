//! A mouse drag over the terminal selects (BB-BUG-053): press starts, move
//! extends, release ends; a click without a drag clears it.

use gpui::{
    div, prelude::*, Context, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels, Point, Window,
};

use super::super::Cell;
use crate::ExplorerView;

/// The rows' box: it records where it is and takes the mouse.
pub(super) fn grid(
    view: &ExplorerView,
    cx: &mut Context<ExplorerView>,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id("terminal-grid")
        .debug_selector(|| "terminal-grid".to_owned())
        .track_scroll(&view.terminal.grid)
        .flex()
        .flex_col()
        .cursor_text()
        .on_mouse_down(MouseButton::Left, cx.listener(press))
        .on_mouse_move(cx.listener(drag))
        .on_scroll(cx.listener(wheel))
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(
                |view: &mut ExplorerView,
                 _: &gpui::MouseUpEvent,
                 _: &mut Window,
                 cx: &mut Context<ExplorerView>| {
                    view.terminal.selecting = false;
                    cx.notify();
                },
            ),
        )
}

fn press(
    view: &mut ExplorerView,
    event: &MouseDownEvent,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    window.focus(&view.terminal.focus, cx);
    let cell: Cell = cell_at(view, event.position, window, cx);
    if let Some(session) = view.terminal.session.as_ref() {
        session.select_from(cell);
        view.terminal.selecting = true;
        cx.notify();
    }
}

fn drag(
    view: &mut ExplorerView,
    event: &MouseMoveEvent,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    if !view.terminal.selecting || event.pressed_button != Some(MouseButton::Left) {
        view.terminal.selecting = false;
        return;
    }
    let cell: Cell = cell_at(view, event.position, window, cx);
    if let Some(session) = view.terminal.session.as_ref() {
        session.select_to(cell);
        cx.notify();
    }
}

/// BB-BUG-058: the wheel scrolls the history by whole lines; a trackpad's
/// small steps add up in `scroll_rest` until they make one.
fn wheel(
    view: &mut ExplorerView,
    event: &gpui::ScrollEvent,
    _: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let height: f32 = crate::settings::typography(cx).line_height().max(1.0);
    let rest: f32 =
        view.terminal.scroll_rest + f32::from(event.delta.pixel_delta(gpui::px(height)).y);
    let (count, left): (usize, f32) = steps(rest.abs(), height);
    view.terminal.scroll_rest = left.copysign(rest);
    let lines: i32 = i32::try_from(count).unwrap_or(i32::MAX);
    if let (Some(session), true) = (view.terminal.session.as_ref(), lines > 0) {
        session.scroll(if rest > 0.0 { lines } else { -lines });
        cx.notify();
    }
    cx.stop_propagation();
}

/// The screen cell at `at`, from last frame's grid box and the font.
fn cell_at(
    view: &ExplorerView,
    at: Point<Pixels>,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) -> Cell {
    let origin: Point<Pixels> = view.terminal.grid.bounds().origin;
    let fonts: bb_ui::Typography = crate::settings::typography(cx);
    let width: f32 = crate::editor::Columns::shape(window, "M", &fonts)
        .x_of(1)
        .max(1.0);
    let (column, rest): (usize, f32) = steps(f32::from(at.x - origin.x), width);
    let (row, _): (usize, f32) = steps(f32::from(at.y - origin.y), fonts.line_height());
    Cell {
        row,
        column,
        right_half: rest >= width / 2.0,
    }
}

/// Whole `step`s in `space` (at least 0) and what is left (no float casts).
fn steps(space: f32, step: f32) -> (usize, f32) {
    let (mut count, mut left): (usize, f32) = (0, space.max(0.0));
    while left >= step && count < 1_000 {
        count += 1;
        left -= step;
    }
    (count, left)
}
