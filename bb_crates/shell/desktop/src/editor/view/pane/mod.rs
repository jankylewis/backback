//! The editor panes (BB-BUG-040): one, or up to three side by side
//! (BB-BUG-045) as tabs are dragged to the right half of the last pane (or
//! moved with Ctrl+Cmd+Right), as in VS Code. The focused pane is the live
//! editor; another draws its group's active tab and takes focus on the
//! first press, which then acts as usual.

pub(crate) mod bb_hscroll;
pub(crate) mod drag;

use gpui::{div, prelude::*, px, relative, uniform_list, Context, IntoElement, Window};

use super::super::bb_tabs::bb_view::{tab_bar, DraggedTab};
use super::super::{Group, MAX_GROUPS};
use crate::{color, ExplorerView};

impl ExplorerView {
    /// The focused pane's left edge in the window: after the tool window
    /// (and its 6 px edge when it docks left), the panes left of it and the
    /// lines between them (BB-BUG-040, BB-BUG-045).
    pub(crate) fn editor_left(&self, fonts: &bb_ui::Typography) -> f32 {
        let before: f32 = self
            .lines
            .widths
            .iter()
            .take(self.tabs.focused().0)
            .map(|w: &f32| w + DIVIDER_PX)
            .sum();
        fonts.editor_x(self.panel_width) - fonts.gutter_width() + before
    }
}

/// The line between two panes.
pub(crate) const DIVIDER_PX: f32 = drag::HANDLE_PX;

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

/// Size this frame's panes from each one's share of the editor area, as the
/// lines between them were dragged (BB-BUG-044); a change in the number of
/// panes shares the area evenly again. Click and scroll maths read them.
pub(crate) fn layout(view: &mut ExplorerView, window: &Window, cx: &gpui::App) {
    let fonts: bb_ui::Typography = crate::settings::typography(cx);
    let count: usize = view.tabs.groups();
    let gaps: f32 = f32::from(u16::try_from(count.saturating_sub(1)).unwrap_or(0));
    let room: f32 = (area(view, window, &fonts) - gaps * DIVIDER_PX).max(0.0);
    view.lines.share_for(count);
    view.lines.widths = std::array::from_fn(|g: usize| {
        if g < count {
            (room * view.lines.shares.get(g).copied().unwrap_or(0.0)).floor()
        } else {
            0.0
        }
    });
}

pub(crate) fn panes(view: &ExplorerView, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    let count: usize = view.tabs.groups();
    let mut root: gpui::Div = div().flex_1().min_h(px(0.0)).flex().overflow_hidden();
    for g in 0..count {
        if g > 0 {
            root = root.child(
                drag::handle(
                    drag::Edge::Split(g - 1),
                    DIVIDERS.get(g - 1).copied().unwrap_or("pane-divider"),
                    view.theme,
                    cx,
                )
                .w(px(DIVIDER_PX)),
            );
        }
        let last: bool = g + 1 == count;
        let width: Option<f32> = (!last).then(|| view.lines.widths.get(g).copied().unwrap_or(0.0));
        root = root.child(pane(view, Group(g), width, cx));
    }
    root
}

/// The selector of the line right of each pane but the last.
const DIVIDERS: [&str; MAX_GROUPS - 1] = ["pane-divider-0", "pane-divider-1"];

fn pane(
    view: &ExplorerView,
    group: Group,
    width: Option<f32>,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    let body: gpui::AnyElement = if group == view.tabs.focused() {
        super::editor_body(view, cx).into_any_element()
    } else {
        other_body(view, group, cx).into_any_element()
    };
    div()
        .debug_selector(move || format!("pane-{}", group.0))
        .h_full()
        .min_w(px(0.0))
        .relative()
        .flex()
        .flex_col()
        .map(|d: gpui::Div| match width {
            Some(w) => d.w(px(w)).flex_none(),
            None => d.flex_1(),
        })
        .child(tab_bar(&view.tabs, group, view.theme, cx))
        .child(body)
        .children(drop_zone(view, group, cx))
}

/// A pane that is not focused: its active tab's lines; the first press
/// focuses it before the line under it handles the press.
fn other_body(
    view: &ExplorerView,
    group: Group,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    let lines: usize = view
        .tabs
        .active_tab_of(group)
        .map_or(0, |tab: &crate::editor::Tab| tab.document.len_lines());
    div()
        .id(("pane-body", group.0))
        .debug_selector(move || format!("pane-body-{}", group.0))
        .flex_1()
        .bg(color(view.theme.bg))
        .capture_any_mouse_down(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::MouseDownEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                view.tabs.focus(group);
                window.focus(&view.editor_focus, cx);
                cx.notify();
            },
        ))
        .child(
            uniform_list(
                ("pane-lines", group.0),
                lines,
                cx.processor(
                    move |view: &mut ExplorerView,
                          range: std::ops::Range<usize>,
                          window: &mut Window,
                          row_cx: &mut Context<ExplorerView>| {
                        range
                            .map(|index: usize| {
                                super::editor_line(view, group, index, window, row_cx)
                            })
                            .collect::<Vec<_>>()
                    },
                ),
            )
            .track_scroll(view.tabs.scroll_of(group))
            .h_full(),
        )
}

/// While a tab drags, each pane takes it into its group; the right half of
/// the last pane makes a new group, while fewer than three show.
fn drop_zone(
    view: &ExplorerView,
    group: Group,
    cx: &mut Context<ExplorerView>,
) -> Option<impl IntoElement> {
    if !cx.has_active_drag() {
        return None;
    }
    let count: usize = view.tabs.groups();
    let grows: bool = group.0 + 1 == count && count < MAX_GROUPS;
    let accent: gpui::Rgba = color(view.theme.accent);
    let own: gpui::Stateful<gpui::Div> = drop_target(group, 1.0, accent, cx);
    let new: Option<gpui::Stateful<gpui::Div>> =
        grows.then(|| drop_target(group.right(), 0.5, accent, cx));
    Some(div().absolute().size_full().child(own).children(new))
}

/// A drop target over the right `share` of a pane: the tab goes to `target`.
fn drop_target(
    target: Group,
    share: f32,
    accent: gpui::Rgba,
    cx: &mut Context<ExplorerView>,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(("drop", target.0))
        .debug_selector(move || format!("drop-{}", target.0))
        .absolute()
        .top_0()
        .bottom_0()
        .right_0()
        .w(relative(share))
        .drag_over::<DraggedTab>(
            move |style: gpui::StyleRefinement,
                  _: &DraggedTab,
                  _: &mut Window,
                  _: &mut gpui::App| { style.bg(gpui::Rgba { a: 0.25, ..accent }) },
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
        ))
}

#[cfg(test)]
pub(crate) mod lines_tests;
#[cfg(test)]
pub(crate) mod window_tests;
