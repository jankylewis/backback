//! Lines a press can drag (BB-BUG-044), as in VS Code: the tool window's
//! edge, the line between the two editor panes, and the top edge of the
//! panel under the editor (Find in Files, references, Problems). A drag
//! moves from where it started, so a terminal or status bar never skews it.

use gpui::{div, prelude::*, px, Context, MouseButton, Window};

use crate::editor::MAX_GROUPS;
use crate::{color, ExplorerView};

/// Which line is being dragged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Edge {
    /// The tool window's edge: its width.
    Panel,
    /// The line right of pane `n`: its share and its right neighbour's.
    Split(usize),
    /// The top of the panel under the editor: its height.
    Bottom,
    /// The top of the terminal: its height (BB-BUG-046).
    Terminal,
}

/// A drag under way: the line, the press point (x or y), the size then,
/// and for a line between panes the two panes' width together.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Drag {
    edge: Edge,
    at: f32,
    size: f32,
    pair: f32,
}

/// The sizes the lines set, and the drag under way.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Lines {
    pub(crate) dragging: Option<Drag>,
    /// Each pane's share of the panes' width (BB-BUG-045).
    pub(crate) shares: [f32; MAX_GROUPS],
    /// How many panes `shares` is for; another count shares evenly again.
    shares_for: usize,
    /// Each pane's width, set as the window draws.
    pub(crate) widths: [f32; MAX_GROUPS],
    /// The height of the panel under the editor.
    pub(crate) bottom_px: f32,
    /// The height of the terminal.
    pub(crate) terminal_px: f32,
    /// A scroll bar's thumb held (BB-BUG-051).
    pub(crate) bar: Option<super::super::bb_scrollbar::BarDrag>,
}

impl Default for Lines {
    fn default() -> Self {
        Self {
            dragging: None,
            shares: [1.0, 0.0, 0.0],
            shares_for: 1,
            widths: [0.0; MAX_GROUPS],
            bottom_px: 200.0,
            terminal_px: 240.0,
            bar: None,
        }
    }
}

impl Lines {
    /// Share the area evenly when the number of panes changed.
    pub(crate) fn share_for(&mut self, count: usize) {
        if count == self.shares_for || count == 0 {
            return;
        }
        let even: f32 = 1.0 / f32::from(u16::try_from(count).unwrap_or(1));
        self.shares = std::array::from_fn(|g: usize| if g < count { even } else { 0.0 });
        self.shares_for = count;
    }

    /// The height a top edge sets: the terminal's or the bottom panel's.
    fn height_mut(&mut self, edge: Edge) -> &mut f32 {
        if edge == Edge::Terminal {
            &mut self.terminal_px
        } else {
            &mut self.bottom_px
        }
    }

    fn width(&self, pane: usize) -> f32 {
        self.widths.get(pane).copied().unwrap_or(0.0)
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
        Edge::Panel | Edge::Split(_) => line.h_full().cursor_col_resize(),
        Edge::Bottom | Edge::Terminal => line.w_full().cursor_row_resize(),
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
                    let (x, y): (f32, f32) =
                        (f32::from(event.position.x), f32::from(event.position.y));
                    let lines: &Lines = &view.lines;
                    let (at, size, pair): (f32, f32, f32) = match edge {
                        Edge::Panel => (x, view.panel_width, 0.0),
                        Edge::Split(n) => (
                            x,
                            lines.width(n),
                            lines.width(n) + lines.width(n.saturating_add(1)),
                        ),
                        Edge::Bottom => (y, lines.bottom_px, 0.0),
                        Edge::Terminal => (y, lines.terminal_px, 0.0),
                    };
                    view.lines.dragging = Some(Drag {
                        edge,
                        at,
                        size,
                        pair,
                    });
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
        if super::super::bb_scrollbar::drag(self, event, cx) {
            return;
        }
        let Some(Drag {
            edge,
            at: from,
            size,
            pair,
        }): Option<Drag> = self.lines.dragging
        else {
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
            Edge::Split(n) => {
                let room: f32 = self.lines.widths.iter().sum::<f32>().max(1.0);
                let left: f32 = (size + x - from).clamp(MIN_PX, (pair - MIN_PX).max(MIN_PX));
                if let Some(share) = self.lines.shares.get_mut(n) {
                    *share = left / room;
                }
                if let Some(share) = self.lines.shares.get_mut(n.saturating_add(1)) {
                    *share = (pair - left) / room;
                }
            }
            Edge::Bottom | Edge::Terminal => {
                let tallest: f32 = f32::from(window.viewport_size().height) * 0.8;
                *self.lines.height_mut(edge) = (size + from - y).clamp(MIN_PX, tallest.max(MIN_PX));
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
        self.lines.bar = None;
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
