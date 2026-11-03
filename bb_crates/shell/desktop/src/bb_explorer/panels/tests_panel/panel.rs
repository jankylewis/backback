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
        .child(side_tabs(crate::tests_panel::Side::Files, theme, cx))
}

/// EXPLORER and TESTS side by side, as the headmate asked (2026-11-03): the
/// chosen one in pink, the other one a link to its panel.
pub(crate) fn side_tabs(
    side: crate::tests_panel::Side,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    use crate::tests_panel::Side;
    div()
        .flex()
        .gap_4()
        .child(tab(
            ("open-files", "EXPLORER"),
            side == Side::Files,
            theme,
            cx,
            |view: &mut ExplorerView, _: &mut Window, cx: &mut Context<ExplorerView>| {
                view.tests.side = Side::Files;
                cx.notify();
            },
        ))
        .child(tab(
            ("open-tests", "TESTS"),
            side == Side::Tests,
            theme,
            cx,
            crate::tests_panel::open,
        ))
}

/// One tab: a click on the other tab switches the panel; on the chosen one
/// it does nothing.
fn tab(
    (id, label): (&'static str, &'static str),
    chosen: bool,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
    switch: impl Fn(&mut ExplorerView, &mut Window, &mut Context<ExplorerView>) + 'static,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .debug_selector(move || String::from(id))
        .cursor_pointer()
        .text_color(color(tab_color(chosen, theme)))
        .child(label)
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                cx.stop_propagation();
                if !chosen {
                    switch(view, window, cx);
                }
            },
        ))
}

/// Pink for the chosen tab, the text colour for the other one.
fn tab_color(chosen: bool, theme: Theme) -> bb_ui::bb_theme::Rgb {
    if chosen {
        theme.chosen
    } else {
        theme.text
    }
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

#[cfg(test)]
mod tests {
    use gpui::{Bounds, Modifiers, Pixels, TestAppContext, VisualTestContext};

    use super::tab_color;
    use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
    use crate::tests_panel::Side;
    use crate::ExplorerView;

    #[test]
    fn the_chosen_tab_is_pink_and_the_other_one_takes_the_text_colour() {
        let theme: bb_ui::bb_theme::Theme = bb_ui::bb_theme::LIGHT;
        assert_eq!(tab_color(true, theme), theme.chosen);
        assert_eq!(tab_color(false, theme), theme.text);
    }

    fn side(rig: &Rig, vcx: &mut VisualTestContext) -> Side {
        rig.view
            .read_with(vcx, |v: &ExplorerView, _: &gpui::App| v.tests.side)
    }

    #[gpui::test]
    fn explorer_and_tests_sit_side_by_side_and_switch_the_panel(cx: &mut TestAppContext) {
        let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
        let files: Bounds<Pixels> = vcx.debug_bounds("open-files").unwrap();
        let tests: Bounds<Pixels> = vcx.debug_bounds("open-tests").unwrap();
        assert!(
            tests.left() > files.right() && tests.left() - files.right() < gpui::px(40.0),
            "TESTS right next to EXPLORER: {files:?} {tests:?}"
        );
        vcx.simulate_click(tests.center(), Modifiers::default());
        assert_eq!(side(&rig, vcx), Side::Tests);
        let files: Bounds<Pixels> = vcx
            .debug_bounds("open-files")
            .expect("the tabs on the Tests side too");
        vcx.simulate_click(files.center(), Modifiers::default());
        assert_eq!(side(&rig, vcx), Side::Files);
    }
}
