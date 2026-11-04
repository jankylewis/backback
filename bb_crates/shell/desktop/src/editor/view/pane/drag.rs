//! Lines a press can drag (BB-BUG-044), as in VS Code: the tool window's
//! edge, the line between the two editor panes, and the top edge of the
//! panel under the editor (Find in Files, references, Problems). A drag
//! moves from where it started, so a terminal or status bar never skews it.

use gpui::{div, prelude::*, px, Context, MouseButton, Window};

use crate::{color, ExplorerView};

/// Which line is being dragged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Edge {
    /// The tool window's edge: its width.
    Panel,
    /// The line between the panes: the left pane's share.
    Split,
    /// The top of the panel under the editor: its height.
    Bottom,
}

/// The sizes the lines set, and the drag under way.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Lines {
    /// The line being dragged, the press point (x or y) and the size then.
    pub(crate) dragging: Option<(Edge, f32, f32)>,
    /// The left pane's share of the two panes' width.
    pub(crate) split: f32,
    /// The height of the panel under the editor.
    pub(crate) bottom_px: f32,
    /// The right pane's width while split, set as the window draws.
    pub(crate) right_px: f32,
}

impl Default for Lines {
    fn default() -> Self {
        Self {
            dragging: None,
            split: 0.5,
            bottom_px: 200.0,
            right_px: 0.0,
        }
    }
}

/// The smallest pane or panel a drag leaves.
const MIN_PX: f32 = 80.0;

/// A line a press starts to drag: `wide` for a line between columns.
pub(crate) fn handle(
    edge: Edge,
    name: &'static str,
    theme: bb_ui::bb_theme::Theme,
    cx: &mut Context<ExplorerView>,
) -> gpui::Stateful<gpui::Div> {
    let line: gpui::Div = div().flex_shrink_0().bg(color(theme.border));
    let line: gpui::Div = match edge {
        Edge::Panel | Edge::Split => line.h_full().cursor_col_resize(),
        Edge::Bottom => line.w_full().cursor_row_resize(),
    };
    line.id(name)
        .debug_selector(move || name.to_owned())
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(
                move |view: &mut ExplorerView,
                      event: &gpui::MouseDownEvent,
                      _: &mut Window,
                      cx: &mut Context<ExplorerView>| {
                    let (at, size): (f32, f32) = match edge {
                        Edge::Panel => (f32::from(event.position.x), view.panel_width),
                        Edge::Split => (f32::from(event.position.x), view.tabs.pane_px),
                        Edge::Bottom => (f32::from(event.position.y), view.lines.bottom_px),
                    };
                    view.lines.dragging = Some((edge, at, size));
                    cx.stop_propagation();
                },
            ),
        )
}

impl ExplorerView {
    pub(crate) fn move_resize(
        &mut self,
        event: &gpui::MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some((edge, from, size)): Option<(Edge, f32, f32)> = self.lines.dragging else {
            return;
        };
        if !event.dragging() {
            self.lines.dragging = None;
            return;
        }
        let (x, y): (f32, f32) = (f32::from(event.position.x), f32::from(event.position.y));
        let fonts: bb_ui::Typography = crate::settings::typography(cx);
        match edge {
            Edge::Panel => {
                let moved: f32 = if fonts.panel_right {
                    from - x
                } else {
                    x - from
                };
                self.panel_width = (size + moved).clamp(180.0, 800.0);
            }
            Edge::Split => {
                let room: f32 = super::area(self, window, &fonts) - super::DIVIDER_PX;
                let left: f32 = (size + x - from).clamp(MIN_PX, (room - MIN_PX).max(MIN_PX));
                self.lines.split = (left / room.max(1.0)).clamp(0.0, 1.0);
            }
            Edge::Bottom => {
                let tallest: f32 = f32::from(window.viewport_size().height) * 0.8;
                self.lines.bottom_px = (size + from - y).clamp(MIN_PX, tallest.max(MIN_PX));
            }
        }
        cx.notify();
    }

    pub(crate) fn finish_resize(
        &mut self,
        _: &gpui::MouseUpEvent,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
        self.lines.dragging = None;
    }
}

/// The handle's thickness, in points.
pub(crate) const HANDLE_PX: f32 = 4.0;

/// The top edge of the panel under the editor, to drag its height.
pub(crate) fn bottom_handle(
    theme: bb_ui::bb_theme::Theme,
    cx: &mut Context<ExplorerView>,
) -> gpui::Stateful<gpui::Div> {
    handle(Edge::Bottom, "bottom-panel-edge", theme, cx).h(px(HANDLE_PX))
}
