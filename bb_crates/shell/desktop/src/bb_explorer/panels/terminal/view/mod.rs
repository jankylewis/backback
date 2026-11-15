//! The terminal panel: one line per row in the editor's font and the
//! theme's text colour (BB-BUG-046), and a block cursor; keys go to the
//! shell, Cmd+V pastes. Its top edge drags its height; the grid follows.
//! A mouse drag selects (BB-BUG-053), drawn in the theme's `selection`;
//! Cmd+C copies it. Split panes (BB-FEAT-079) share the width equally,
//! with a one-pixel line between them.

use gpui::{div, prelude::*, px, AnyElement, Context, Window};

mod bb_mouse;
mod keys;
mod rows;

use super::bb_session::Screen;
use crate::{color, ExplorerView};

/// Pixels between two panes (the dividing line).
const DIVIDER_PX: f32 = 1.0;

/// Columns and rows that fit one of the panes, from the font and the editor
/// area's width (the tool window left out on either side, BB-BUG-056).
fn grid(view: &ExplorerView, window: &Window, fonts: &bb_ui::Typography) -> (usize, usize) {
    let width: f32 = crate::editor::editor_area(view, window, fonts);
    let count: usize = view.terminal.panes.len().max(1);
    let share: f32 = (width - DIVIDER_PX * (steps(count) - 1.0)) / steps(count);
    let cell: f32 = crate::editor::Columns::shape(window, "M", fonts)
        .x_of(1)
        .max(1.0);
    (
        fit(share - 16.0, cell),
        fit(
            view.lines.terminal_px - 12.0 - crate::editor::HANDLE_PX - crate::intel::view::TITLE_PX,
            fonts.line_height(),
        ),
    )
}

/// `count` as a float (no int-to-float casts; panes are few).
fn steps(count: usize) -> f32 {
    f32::from(u16::try_from(count).unwrap_or(u16::MAX))
}

/// How many `step`s fit in `space` (no float-to-int casts).
fn fit(space: f32, step: f32) -> usize {
    let mut count: usize = 0;
    let mut used: f32 = step;
    while used <= space && count < 1_000 {
        count += 1;
        used += step;
    }
    count.max(1)
}

/// The terminal's text and background: the editor's (BB-BUG-046). Before,
/// no text colour was set and rows drew black on the dark theme.
pub(super) fn colors(theme: bb_ui::Theme) -> (bb_ui::Rgb, bb_ui::Rgb) {
    (theme.text, theme.bg)
}

/// The panel when open; also fits each shell's grid to its pane.
pub(crate) fn panel(
    view: &mut ExplorerView,
    window: &Window,
    cx: &mut Context<ExplorerView>,
) -> Option<AnyElement> {
    if !view.terminal.open || view.terminal.panes.is_empty() {
        return None;
    }
    let fonts: bb_ui::Typography = crate::settings::typography(cx);
    let (columns, lines): (usize, usize) = grid(view, window, &fonts);
    let theme: bb_ui::Theme = view.theme;
    let height: f32 = fonts.line_height();
    let mut panes: Vec<AnyElement> = Vec::new();
    for index in 0..view.terminal.panes.len() {
        if let Some(pane) = view.terminal.panes.get_mut(index) {
            pane.session.resize(columns, lines);
        }
        panes.push(pane(view, index, theme, height, cx));
    }
    Some(
        div()
            .id("terminal")
            .debug_selector(|| "terminal".to_owned())
            .key_context("Terminal")
            .h(px(view.lines.terminal_px))
            .flex_none()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(color(colors(theme).1))
            .text_color(color(colors(theme).0))
            .child(
                crate::editor::handle(crate::editor::Edge::Terminal, "terminal-edge", theme, cx)
                    .h(px(crate::editor::HANDLE_PX)),
            )
            .child(super::panes::title(view, theme, cx))
            .child(div().flex_1().flex().flex_row().children(panes))
            .into_any_element(),
    )
}

/// Pane `index`: its rows in a box that takes its keys and mouse.
fn pane(
    view: &ExplorerView,
    index: usize,
    theme: bb_ui::Theme,
    height: f32,
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    let Some(pane): Option<&super::Pane> = view.terminal.panes.get(index) else {
        return div().into_any_element();
    };
    let screen: Screen = pane.session.screen();
    let rows: Vec<gpui::Div> = rows::drawn(screen, theme, height);
    div()
        .id(("terminal-pane", index))
        .debug_selector(move || format!("terminal-pane-{index}"))
        .track_focus(&pane.focus)
        .on_key_down(cx.listener(
            move |view: &mut ExplorerView,
                  event: &gpui::KeyDownEvent,
                  _: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                keys::key(view, index, event, cx);
            },
        ))
        .flex_1()
        .min_w(px(0.0))
        .overflow_hidden()
        .when(index > 0, |el: gpui::Stateful<gpui::Div>| {
            el.border_l(px(DIVIDER_PX))
                .border_color(color(theme.overlay))
        })
        .p(px(6.0))
        .child(bb_mouse::grid(view, index, cx).children(rows))
        .into_any_element()
}

#[cfg(test)]
mod tests;
