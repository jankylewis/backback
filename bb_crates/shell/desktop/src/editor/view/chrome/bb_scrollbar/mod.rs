//! Scroll bars (BB-BUG-051), as in VS Code: a pane shows one on its right
//! edge while its file is taller than the pane, and the focused pane one on
//! its bottom edge while a line in sight is wider. The thumb shows how much
//! is in sight and where; dragging it scrolls, a press on the track jumps.

use gpui::{point, prelude::*, px, AnyElement, Context, ScrollHandle};

use super::super::super::Group;

#[path = "bb_input.rs"]
mod input;
#[cfg(test)]
mod tests;
use crate::ExplorerView;
pub(crate) use input::drag;
use input::{knob, rail};

/// A bar's thickness.
pub(crate) const BAR_PX: f32 = 10.0;
/// The shortest thumb, so a huge file still has one to grab.
const MIN_THUMB_PX: f32 = 24.0;

/// Which way a bar scrolls: a pane's lines down, or the focused text across.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Axis {
    Down(Group),
    Across,
}

/// A thumb on its track: length and start in px, how far the content
/// scrolls for each px the thumb moves, and the farthest scroll.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Thumb {
    pub(crate) length: f32,
    pub(crate) start: f32,
    pub(crate) per_px: f32,
    max: f32,
}

/// A thumb drag under way: the press point and the scroll then.
#[derive(Clone, Copy, Debug)]
pub(crate) struct BarDrag {
    axis: Axis,
    from: f32,
    start: f32,
    thumb: Thumb,
}

/// The thumb for a `track` px long over content that scrolls `max` px past
/// it, now `scrolled` px in; `None` when all of it fits.
pub(crate) fn thumb(track: f32, max: f32, scrolled: f32) -> Option<Thumb> {
    if max < 1.0 || track <= MIN_THUMB_PX {
        return None;
    }
    let length: f32 = (track * track / (track + max)).clamp(MIN_THUMB_PX, track);
    let room: f32 = (track - length).max(1.0);
    Some(Thumb {
        length,
        start: scrolled.clamp(0.0, max) / max * room,
        per_px: max / room,
        max,
    })
}

/// The lines' scroll of `group`, as its list measured it last frame.
fn lines_scroll(view: &ExplorerView, group: Group) -> ScrollHandle {
    view.tabs.scroll_of(group).0.borrow().base_handle.clone()
}

/// The track's start in the window and the bar's thumb, if it needs one.
fn measure(view: &ExplorerView, axis: Axis, cx: &gpui::App) -> Option<(f32, Thumb)> {
    match axis {
        Axis::Down(group) => {
            let scroll: ScrollHandle = lines_scroll(view, group);
            let bounds: gpui::Bounds<gpui::Pixels> = scroll.bounds();
            let found: Thumb = thumb(
                f32::from(bounds.size.height),
                f32::from(scroll.max_offset().y),
                -f32::from(scroll.offset().y),
            )?;
            Some((f32::from(bounds.origin.y), found))
        }
        Axis::Across => {
            let gutter: f32 = crate::settings::typography(cx).gutter_width();
            let bounds: gpui::Bounds<gpui::Pixels> =
                lines_scroll(view, view.tabs.focused()).bounds();
            let text: f32 = f32::from(bounds.size.width) - gutter;
            let tab: &crate::editor::Tab = view.tabs.active_tab()?;
            let found: Thumb = thumb(text - BAR_PX, tab.widest_px - text, tab.scroll_x)?;
            Some((f32::from(bounds.origin.x) + gutter, found))
        }
    }
}

/// Scroll `axis` to `to` px in, within its content.
fn scroll_to(view: &mut ExplorerView, axis: Axis, to: f32, max: f32) {
    let to: f32 = to.clamp(0.0, max);
    match axis {
        Axis::Down(group) => {
            let scroll: ScrollHandle = lines_scroll(view, group);
            scroll.set_offset(point(scroll.offset().x, px(-to)));
        }
        Axis::Across => {
            if let Some(tab) = view.tabs.active_tab_mut() {
                tab.scroll_x = to;
            }
        }
    }
}

/// A pane's bars, over its lines: right edge, and bottom for the focused one.
pub(crate) fn bars(
    view: &ExplorerView,
    group: Group,
    cx: &mut Context<ExplorerView>,
) -> Vec<AnyElement> {
    let mut axes: Vec<Axis> = vec![Axis::Down(group)];
    if group == view.tabs.focused() {
        axes.push(Axis::Across);
    }
    axes.into_iter()
        .filter_map(|axis: Axis| {
            measure(view, axis, cx).map(|(_, t): (f32, Thumb)| bar(view, axis, t, cx))
        })
        .collect()
}

fn names(axis: Axis) -> (&'static str, &'static str) {
    match axis {
        Axis::Down(Group(0)) => ("vbar-0", "vbar-thumb-0"),
        Axis::Down(Group(1)) => ("vbar-1", "vbar-thumb-1"),
        Axis::Down(_) => ("vbar-2", "vbar-thumb-2"),
        Axis::Across => ("hbar", "hbar-thumb"),
    }
}

fn bar(
    view: &ExplorerView,
    axis: Axis,
    found: Thumb,
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    let (track, grip): (&'static str, &'static str) = names(axis);
    let knob: gpui::Stateful<gpui::Div> = knob(grip, axis, view.theme, cx);
    match axis {
        Axis::Down(_) => rail(track, axis, cx)
            .top_0()
            .right_0()
            .w(px(BAR_PX))
            .h_full()
            .child(
                knob.left(px(2.0))
                    .w(px(BAR_PX - 4.0))
                    .top(px(found.start))
                    .h(px(found.length)),
            ),
        Axis::Across => rail(track, axis, cx)
            .bottom_0()
            .left(px(crate::settings::typography(cx).gutter_width()))
            .right(px(BAR_PX))
            .h(px(BAR_PX))
            .child(
                knob.top(px(2.0))
                    .h(px(BAR_PX - 4.0))
                    .left(px(found.start))
                    .w(px(found.length)),
            ),
    }
    .into_any_element()
}
