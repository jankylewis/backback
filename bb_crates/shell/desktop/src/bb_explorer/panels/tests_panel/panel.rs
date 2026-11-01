//! The Explorer panel. Rows are virtualised: only the rows in view (plus the
//! list's overscan) become elements, so a 10,000-entry folder costs the same
//! per frame as a 40-entry one.

use std::ops::Range;

use bb_explorer::Entry;
use bb_ui::bb_theme::Theme;
use gpui::{
    div, prelude::*, px, uniform_list, AnyElement, Context, IntoElement, MouseButton, Window,
};

use crate::bb_row_view::{row, RowState};
use crate::{color, ExplorerView};

pub(crate) fn explorer_panel(
    view: &ExplorerView,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    let count: usize = view.tree.visible().len();
    div()
        .id("explorer-panel")
        .w(px(view.panel_width))
        .flex_none()
        .h_full()
        .flex()
        .flex_col()
        .bg(color(view.theme.surface))
        .key_context("Explorer")
        .track_focus(&view.focus)
        .on_action(cx.listener(ExplorerView::move_up))
        .on_action(cx.listener(ExplorerView::move_down))
        .on_action(cx.listener(ExplorerView::move_left))
        .on_action(cx.listener(ExplorerView::move_right))
        .on_action(cx.listener(ExplorerView::open))
        .child(explorer_header(view.theme, cx))
        .child(
            uniform_list(
                "explorer-rows",
                count,
                cx.processor(
                    |view: &mut ExplorerView,
                     range: Range<usize>,
                     _: &mut Window,
                     row_cx: &mut Context<ExplorerView>| {
                        rows_in(view, range, row_cx)
                    },
                ),
            )
            .track_scroll(&view.explorer_scroll)
            .flex_1(),
        )
}

fn explorer_header(theme: Theme, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    div()
        .px_3()
        .py_2()
        .text_color(color(theme.text))
        .flex()
        .justify_between()
        .child("EXPLORER")
        .child(
            div()
                .id("open-tests")
                .debug_selector(|| String::from("open-tests"))
                .cursor_pointer()
                .text_color(color(theme.accent))
                .child("Tests")
                .on_click(cx.listener(
                    |view: &mut ExplorerView,
                     _: &gpui::ClickEvent,
                     window: &mut Window,
                     cx: &mut Context<ExplorerView>| {
                        crate::tests_panel::open(view, window, cx);
                    },
                )),
        )
}

/// Elements for the visible rows in `range` only.
fn rows_in(
    view: &ExplorerView,
    range: Range<usize>,
    cx: &mut Context<ExplorerView>,
) -> Vec<AnyElement> {
    let rows: Vec<(usize, &Entry)> = view.tree.visible();
    let first: usize = range.start;
    rows.get(range)
        .unwrap_or(&[])
        .iter()
        .enumerate()
        .map(|(offset, (depth, entry)): (usize, &(usize, &Entry))| {
            let index: usize = first.saturating_add(offset);
            let state: RowState = RowState {
                selected: index == view.selected,
                panel_width: view.panel_width,
                theme: view.theme,
            };
            row(index, *depth, entry, state, cx).into_any_element()
        })
        .collect()
}

pub(crate) fn resize_edge(theme: Theme, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    div()
        .id("explorer-resize-edge")
        .w(px(6.0))
        .h_full()
        .flex_shrink_0()
        .bg(color(theme.border))
        .on_mouse_down(MouseButton::Left, cx.listener(ExplorerView::start_resize))
}

impl ExplorerView {
    /// The editor's left edge in the window: after the tool window and its
    /// 6 px edge when it docks left, else the window's left edge.
    pub(crate) fn editor_left(&self, fonts: &bb_ui::Typography) -> f32 {
        fonts.editor_x(self.panel_width) - fonts.gutter_width()
    }

    pub(crate) fn move_resize(
        &mut self,
        event: &gpui::MouseMoveEvent,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        if self.resizing && event.dragging() {
            let x: f32 = f32::from(event.position.x);
            let wide: f32 = if crate::settings::typography(cx).panel_right {
                f32::from(window.viewport_size().width) - x
            } else {
                x
            };
            self.panel_width = wide.clamp(180.0, 800.0);
            cx.notify();
        }
    }

    pub(crate) fn finish_resize(
        &mut self,
        _: &gpui::MouseUpEvent,
        _: &mut gpui::Window,
        _: &mut Context<Self>,
    ) {
        self.resizing = false;
    }

    fn start_resize(
        &mut self,
        _: &gpui::MouseDownEvent,
        _: &mut gpui::Window,
        _: &mut Context<Self>,
    ) {
        self.resizing = true;
    }
}
