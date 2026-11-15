//! A mouse drag over the terminal selects (BB-BUG-053): press starts, move
//! extends, release ends; a click without a drag clears it.

use gpui::{
    div, prelude::*, Context, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels, Point, Window,
};

use super::super::Cell;
use crate::ExplorerView;

/// Pane `index`'s rows box: it records where it is and takes the mouse;
/// the active pane's is `terminal-grid`.
pub(super) fn grid(
    view: &ExplorerView,
    index: usize,
    cx: &mut Context<ExplorerView>,
) -> gpui::Stateful<gpui::Div> {
    let name: String = grid_name(index, view.terminal.active);
    let el: gpui::Stateful<gpui::Div> = div()
        .id(("terminal-grid", index))
        .debug_selector(move || name.clone())
        .flex()
        .flex_col()
        .cursor_text();
    let el: gpui::Stateful<gpui::Div> = match view.terminal.panes.get(index) {
        Some(pane) => el.track_scroll(&pane.grid),
        None => el,
    };
    el.on_mouse_down(
        MouseButton::Left,
        cx.listener(
            move |view: &mut ExplorerView,
                  event: &MouseDownEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| press(view, index, event, window, cx),
        ),
    )
    .on_mouse_move(cx.listener(
        move |view: &mut ExplorerView,
              event: &MouseMoveEvent,
              window: &mut Window,
              cx: &mut Context<ExplorerView>| drag(view, index, event, window, cx),
    ))
    .on_scroll(cx.listener(
        move |view: &mut ExplorerView,
              event: &gpui::ScrollEvent,
              _: &mut Window,
              cx: &mut Context<ExplorerView>| wheel(view, index, event, cx),
    ))
    .on_mouse_up(
        MouseButton::Left,
        cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::MouseUpEvent,
                  _: &mut Window,
                  cx: &mut Context<ExplorerView>| release(view, index, cx),
        ),
    )
}

/// `terminal-grid` for the active pane, `terminal-grid-<n>` for the rest.
fn grid_name(index: usize, active: usize) -> String {
    if index == active {
        "terminal-grid".to_owned()
    } else {
        format!("terminal-grid-{index}")
    }
}

/// The button is up: the selection is done.
fn release(view: &mut ExplorerView, index: usize, cx: &mut Context<ExplorerView>) {
    if let Some(pane) = view.terminal.panes.get_mut(index) {
        pane.selecting = false;
    }
    cx.notify();
}

/// A press focuses the pane (it takes the keys) and starts a selection.
fn press(
    view: &mut ExplorerView,
    index: usize,
    event: &MouseDownEvent,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let cell: Cell = cell_at(view, index, event.position, window, cx);
    let Some(pane): Option<&mut super::super::Pane> = view.terminal.panes.get_mut(index) else {
        return;
    };
    window.focus(&pane.focus, cx);
    pane.session.select_from(cell);
    pane.selecting = true;
    view.terminal.active = index;
    cx.notify();
}

fn drag(
    view: &mut ExplorerView,
    index: usize,
    event: &MouseMoveEvent,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let cell: Cell = cell_at(view, index, event.position, window, cx);
    let Some(pane): Option<&mut super::super::Pane> = view.terminal.panes.get_mut(index) else {
        return;
    };
    if !pane.selecting || event.pressed_button != Some(MouseButton::Left) {
        pane.selecting = false;
        return;
    }
    pane.session.select_to(cell);
    cx.notify();
}

/// BB-BUG-058: the wheel scrolls the history by whole lines; a trackpad's
/// small steps add up in `scroll_rest` until they make one.
fn wheel(
    view: &mut ExplorerView,
    index: usize,
    event: &gpui::ScrollEvent,
    cx: &mut Context<ExplorerView>,
) {
    cx.stop_propagation();
    let height: f32 = crate::settings::typography(cx).line_height().max(1.0);
    let Some(pane): Option<&mut super::super::Pane> = view.terminal.panes.get_mut(index) else {
        return;
    };
    let rest: f32 = pane.scroll_rest + f32::from(event.delta.pixel_delta(gpui::px(height)).y);
    let (count, left): (usize, f32) = steps(rest.abs(), height);
    pane.scroll_rest = left.copysign(rest);
    let lines: i32 = i32::try_from(count).unwrap_or(i32::MAX);
    if lines > 0 {
        pane.session.scroll(if rest > 0.0 { lines } else { -lines });
        cx.notify();
    }
}

/// The screen cell at `at`, from last frame's grid box and the font.
fn cell_at(
    view: &ExplorerView,
    index: usize,
    at: Point<Pixels>,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) -> Cell {
    let origin: Point<Pixels> = view
        .terminal
        .panes
        .get(index)
        .map_or(at, |pane: &super::super::Pane| pane.grid.bounds().origin);
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
