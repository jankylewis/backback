//! The editor panes (BB-BUG-040): one, or two side by side once a tab is
//! dragged to the right half (or moved with Ctrl+Cmd+Right), as in VS Code.
//! The focused pane is the live editor; the other draws its group's active
//! tab and takes focus on the first press, which then acts as usual.

pub(crate) mod bb_hscroll;
pub(crate) mod drag;
pub(crate) mod test_marks;

use gpui::{div, prelude::*, px, relative, uniform_list, Context, IntoElement, Window};

use super::super::bb_tabs::bb_view::{tab_bar, DraggedTab};
use super::super::Side;
use crate::{color, ExplorerView};

impl ExplorerView {
    /// The editor's left edge in the window: after the tool window and its
    /// 6 px edge when it docks left, else the window's left edge.
    /// With two editor panes, the focused right pane starts after the left
    /// pane and the line between them (BB-BUG-040).
    pub(crate) fn editor_left(&self, fonts: &bb_ui::Typography) -> f32 {
        let right_pane: f32 = if self.tabs.focused() == crate::editor::Side::Right {
            self.tabs.pane_px + crate::editor::DIVIDER_PX
        } else {
            0.0
        };
        fonts.editor_x(self.panel_width) - fonts.gutter_width() + right_pane
    }
}

/// The line between the two panes.
pub(crate) const DIVIDER_PX: f32 = drag::HANDLE_PX;

fn name(side: Side) -> &'static str {
    match side {
        Side::Left => "left",
        Side::Right => "right",
    }
}

/// The editor area's width: the window less the tool window and its edge.
pub(crate) fn area(view: &ExplorerView, window: &Window, fonts: &bb_ui::Typography) -> f32 {
    let right_dock: f32 = if fonts.panel_right {
        view.panel_width + 6.0
    } else {
        0.0
    };
    let left: f32 = fonts.editor_x(view.panel_width) - fonts.gutter_width();
    f32::from(window.viewport_size().width) - left - right_dock
}

/// Size this frame's panes: while split, the left one takes its share of
/// the editor area, as the line between them was dragged (BB-BUG-044); the
/// click and scroll maths read both widths (BB-BUG-040).
pub(crate) fn layout(view: &mut ExplorerView, window: &Window, cx: &gpui::App) {
    let fonts: bb_ui::Typography = crate::settings::typography(cx);
    let room: f32 = (area(view, window, &fonts) - DIVIDER_PX).max(0.0);
    let left: f32 = if view.tabs.is_split() {
        (room * view.lines.split).floor()
    } else {
        0.0
    };
    view.tabs.pane_px = left;
    view.lines.right_px = room - left;
}

pub(crate) fn panes(view: &ExplorerView, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    let root: gpui::Div = div().flex_1().min_h(px(0.0)).flex().overflow_hidden();
    if !view.tabs.is_split() {
        return root.child(pane(view, Side::Left, None, cx));
    }
    root.child(pane(view, Side::Left, Some(view.tabs.pane_px), cx))
        .child(drag::handle(drag::Edge::Split, "pane-divider", view.theme, cx).w(px(DIVIDER_PX)))
        .child(pane(view, Side::Right, None, cx))
}

fn pane(
    view: &ExplorerView,
    side: Side,
    width: Option<f32>,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    let body: gpui::AnyElement = if side == view.tabs.focused() {
        super::editor_body(view, cx).into_any_element()
    } else {
        other_body(view, side, cx).into_any_element()
    };
    div()
        .debug_selector(move || format!("pane-{}", name(side)))
        .h_full()
        .min_w(px(0.0))
        .relative()
        .flex()
        .flex_col()
        .map(|d: gpui::Div| match width {
            Some(w) => d.w(px(w)).flex_none(),
            None => d.flex_1(),
        })
        .child(tab_bar(&view.tabs, side, view.theme, cx))
        .child(body)
        .children(drop_zone(view, side, cx))
}

/// The pane that is not focused: its active tab's lines; the first press
/// focuses it before the line under it handles the press.
fn other_body(view: &ExplorerView, side: Side, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    let lines: usize = view
        .tabs
        .active_tab_of(side)
        .map_or(0, |tab: &crate::editor::Tab| tab.document.len_lines());
    div()
        .id(name(side))
        .debug_selector(move || format!("pane-body-{}", name(side)))
        .flex_1()
        .bg(color(view.theme.bg))
        .capture_any_mouse_down(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::MouseDownEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                view.tabs.focus(side);
                window.focus(&view.editor_focus, cx);
                cx.notify();
            },
        ))
        .child(
            uniform_list(
                "pane-lines",
                lines,
                cx.processor(
                    move |view: &mut ExplorerView,
                          range: std::ops::Range<usize>,
                          window: &mut Window,
                          row_cx: &mut Context<ExplorerView>| {
                        range
                            .map(|index: usize| {
                                super::editor_line(view, side, index, window, row_cx)
                            })
                            .collect::<Vec<_>>()
                    },
                ),
            )
            .track_scroll(view.tabs.other_scroll())
            .h_full(),
        )
}

/// While a tab drags: the right half of a lone pane splits the editor; with
/// two panes, each pane takes the tab into its group.
fn drop_zone(
    view: &ExplorerView,
    side: Side,
    cx: &mut Context<ExplorerView>,
) -> Option<impl IntoElement> {
    if !cx.has_active_drag() {
        return None;
    }
    let split: bool = view.tabs.is_split();
    let target: Side = if split { side } else { Side::Right };
    let accent: gpui::Rgba = color(view.theme.accent);
    Some(
        div()
            .id(("drop", usize::from(side == Side::Right)))
            .debug_selector(move || format!("drop-{}", name(target)))
            .absolute()
            .top_0()
            .bottom_0()
            .right_0()
            .w(relative(if split { 1.0 } else { 0.5 }))
            .drag_over::<DraggedTab>(
                move |style: gpui::StyleRefinement,
                      _: &DraggedTab,
                      _: &mut Window,
                      _: &mut gpui::App| {
                    style.bg(gpui::Rgba { a: 0.25, ..accent })
                },
            )
            .on_drop(cx.listener(
                move |view: &mut ExplorerView,
                      dragged: &DraggedTab,
                      window: &mut Window,
                      cx: &mut Context<ExplorerView>| {
                    view.tabs.move_to(dragged.index, target);
                    window.focus(&view.editor_focus, cx);
                    cx.notify();
                },
            )),
    )
}

#[cfg(test)]
pub(crate) mod window_tests;
