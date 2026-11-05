//! A scroll bar's mouse: hold and drag the thumb, or press the track.

use gpui::{div, prelude::*, px, Context, MouseButton, Window};

use super::{measure, scroll_to, Axis, BarDrag};
use crate::{color, ExplorerView};

/// The thumb: a press holds it where the scroll is now.
pub(super) fn knob(
    grip: &'static str,
    axis: Axis,
    theme: bb_ui::bb_theme::Theme,
    cx: &mut Context<ExplorerView>,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(grip)
        .debug_selector(move || grip.to_owned())
        .absolute()
        .rounded(px(3.0))
        .bg(color(theme.border))
        .hover(move |s: gpui::StyleRefinement| s.bg(color(theme.text_dim)))
        .on_mouse_down(MouseButton::Left, cx.listener(hold(axis)))
}

fn hold(
    axis: Axis,
) -> impl Fn(&mut ExplorerView, &gpui::MouseDownEvent, &mut Window, &mut Context<ExplorerView>) + 'static
{
    move |v: &mut ExplorerView,
          e: &gpui::MouseDownEvent,
          _: &mut Window,
          cx: &mut Context<ExplorerView>| {
        cx.stop_propagation();
        if let Some((_, now)) = measure(v, axis, cx) {
            let start: f32 = now.start * now.per_px;
            v.lines.bar = Some(BarDrag {
                axis,
                from: along(axis, e.position),
                start,
                thumb: now,
            });
        }
    }
}

/// The track: a press centres the thumb there.
pub(super) fn rail(
    track: &'static str,
    axis: Axis,
    cx: &mut Context<ExplorerView>,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(track)
        .debug_selector(move || track.to_owned())
        .absolute()
        .on_mouse_down(MouseButton::Left, cx.listener(jump(axis)))
}

fn jump(
    axis: Axis,
) -> impl Fn(&mut ExplorerView, &gpui::MouseDownEvent, &mut Window, &mut Context<ExplorerView>) + 'static
{
    move |v: &mut ExplorerView,
          e: &gpui::MouseDownEvent,
          _: &mut Window,
          cx: &mut Context<ExplorerView>| {
        cx.stop_propagation();
        if let Some((origin, now)) = measure(v, axis, cx) {
            let wanted: f32 = (along(axis, e.position) - origin - now.length / 2.0) * now.per_px;
            scroll_to(v, axis, wanted, now.max);
            cx.notify();
        }
    }
}

/// The pointer's place along `axis`.
fn along(axis: Axis, at: gpui::Point<gpui::Pixels>) -> f32 {
    match axis {
        Axis::Down(_) => f32::from(at.y),
        Axis::Across => f32::from(at.x),
    }
}

/// A pointer move while a thumb is held; `false` when none is.
pub(crate) fn drag(
    view: &mut ExplorerView,
    event: &gpui::MouseMoveEvent,
    cx: &mut Context<ExplorerView>,
) -> bool {
    let Some(held): Option<BarDrag> = view.lines.bar else {
        return false;
    };
    if !event.dragging() {
        view.lines.bar = None;
        return true;
    }
    let moved: f32 = along(held.axis, event.position) - held.from;
    scroll_to(
        view,
        held.axis,
        held.start + moved * held.thumb.per_px,
        held.thumb.max,
    );
    cx.notify();
    true
}
