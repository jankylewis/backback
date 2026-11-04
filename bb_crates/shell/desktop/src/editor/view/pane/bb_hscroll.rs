//! Horizontal scrolling of the editor (BB-BUG-013): a tab keeps how many
//! pixels its text has moved left; the gutter never moves. A cursor move
//! keeps the caret in sight, as VS Code does; a horizontal wheel or trackpad
//! swipe scrolls; nothing scrolls past the longest line on screen.

use gpui::{div, prelude::*, px, Context, IntoElement, ScrollEvent, Window};

use super::super::super::bb_tabs::Tab;
use super::super::bb_line::Columns;
use crate::ExplorerView;

/// Space kept between the caret and the viewport's edges when it scrolls.
const MARGIN: f32 = 24.0;
/// Slack past the longest line's end, in chars.
const SLACK: &str = "MMMMMMMM";

/// The text's window in a row: what is left after the gutter, clipped; the
/// shaped line inside sits `scroll_x` pixels to the left.
pub(crate) fn text_viewport(scroll_x: f32, text: impl IntoElement) -> impl IntoElement {
    div()
        .flex_1()
        .min_w(px(0.0))
        .h_full()
        .overflow_hidden()
        .flex()
        .items_center()
        .child(div().relative().left(px(-scroll_x)).child(text))
}

/// Before the visible rows paint: note the widest line in sight and, when
/// the cursor moved, scroll so its caret is in sight.
pub(crate) fn before_rows(
    view: &mut ExplorerView,
    visible: &std::ops::Range<usize>,
    window: &Window,
    cx: &Context<ExplorerView>,
) {
    let fonts: bb_ui::Typography = crate::settings::typography(cx);
    let viewport: f32 = text_viewport_width(view, window, &fonts);
    let Some(tab): Option<&mut Tab> = view.tabs.active_tab_mut() else {
        return;
    };
    let widest: String = visible
        .clone()
        .filter_map(|index: usize| tab.document.line(index))
        .map(|line: String| line.trim_end_matches(['\r', '\n']).to_owned())
        .max_by_key(|line: &String| line.chars().count())
        .unwrap_or_default();
    tab.widest_px = Columns::shape(window, &widest, &fonts).x_of(widest.chars().count())
        + Columns::shape(window, SLACK, &fonts).x_of(SLACK.len());
    let cursor: bb_text::CharIdx = tab.document.cursor();
    let line: usize = tab.document.cursor_line();
    if tab.sighted == Some(cursor) || !visible.contains(&line) {
        return;
    }
    tab.sighted = Some(cursor);
    let caret: f32 = super::super::caret_x(tab, line, window, &fonts);
    tab.scroll_x = scrolled_to_sight(tab.scroll_x, caret, viewport);
}

/// The scroll that keeps a caret at `caret` (x from the text's left edge)
/// in sight in a viewport `width` wide; it moves only when the caret is out,
/// and a caret that fits in the first viewport shows the line start whole.
fn scrolled_to_sight(scroll_x: f32, caret: f32, width: f32) -> f32 {
    if caret < scroll_x + MARGIN {
        if caret + MARGIN < width {
            0.0
        } else {
            caret - MARGIN
        }
    } else if caret > scroll_x + width - MARGIN {
        (caret - width + MARGIN).max(0.0)
    } else {
        scroll_x
    }
}

/// The width the text may use: the editor's width less the gutter.
fn text_viewport_width(view: &ExplorerView, window: &Window, fonts: &bb_ui::Typography) -> f32 {
    if view.tabs.is_split() {
        return (view.tabs.pane_px - fonts.gutter_width()).max(0.0);
    }
    let total: f32 = f32::from(window.viewport_size().width);
    (total - view.editor_left(fonts) - fonts.gutter_width()).max(0.0)
}

/// A wheel or trackpad move with a horizontal part scrolls the text; the
/// vertical part stays with the line list (never stopped here).
pub(crate) fn on_wheel(
    view: &mut ExplorerView,
    event: &ScrollEvent,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let fonts: bb_ui::Typography = crate::settings::typography(cx);
    let dx: f32 = f32::from(event.delta.pixel_delta(px(fonts.line_height())).x);
    if dx.abs() < 0.01 {
        return;
    }
    let viewport: f32 = text_viewport_width(view, window, &fonts);
    let Some(tab): Option<&mut Tab> = view.tabs.active_tab_mut() else {
        return;
    };
    let limit: f32 = (tab.widest_px - viewport).max(0.0);
    tab.scroll_x = (tab.scroll_x - dx).clamp(0.0, limit);
    cx.notify();
}

#[cfg(test)]
mod tests {
    use gpui::{
        point, px, Bounds, Modifiers, Pixels, ScrollDelta, ScrollEvent, ScrollSource,
        TestAppContext, TouchPhase, VisualTestContext,
    };

    use super::scrolled_to_sight;
    use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
    use crate::ExplorerView;

    #[test]
    fn the_scroll_moves_only_when_the_caret_is_out_of_sight() {
        assert_eq!(scrolled_to_sight(0.0, 100.0, 600.0), 0.0, "in sight: stays");
        assert_eq!(
            scrolled_to_sight(0.0, 900.0, 600.0),
            324.0,
            "past the right edge"
        );
        assert_eq!(
            scrolled_to_sight(324.0, 900.0, 600.0),
            324.0,
            "still in sight"
        );
        assert_eq!(
            scrolled_to_sight(324.0, 10.0, 600.0),
            0.0,
            "left of the view"
        );
        assert_eq!(
            scrolled_to_sight(900.0, 700.0, 600.0),
            676.0,
            "keeps a margin"
        );
        assert_eq!(
            scrolled_to_sight(500.0, 300.0, 600.0),
            0.0,
            "fits from the start"
        );
    }

    fn scroll_x(rig: &Rig, vcx: &mut VisualTestContext) -> f32 {
        rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
            v.tabs.active_tab().unwrap().scroll_x
        })
    }

    fn cursor(rig: &Rig, vcx: &mut VisualTestContext) -> usize {
        rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
            v.tabs.active().unwrap().cursor().raw()
        })
    }

    /// A file with a 600-char line, the cursor at its start (line 2, col 4).
    fn long_line_rig(cx: &mut TestAppContext) -> (Rig, &mut VisualTestContext) {
        let long: String = format!("class A\n{{\n    // {}\n}}\n", "word ".repeat(120));
        let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, &long);
        rig.view.update(
            vcx,
            |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
                let doc: &mut bb_text::Document = v.tabs.active_mut().unwrap();
                let start: usize = doc.line_start(2).unwrap().raw();
                doc.set_cursor(bb_text::CharIdx::new(start + 4)).unwrap();
                cx.notify();
            },
        );
        vcx.run_until_parked();
        (rig, vcx)
    }

    fn swipe(vcx: &mut VisualTestContext, at: Bounds<Pixels>, dx: f32) {
        vcx.simulate_event(ScrollEvent {
            position: at.center(),
            delta: ScrollDelta::Pixels(point(px(dx), px(0.0))),
            modifiers: Modifiers::default(),
            touch_phase: TouchPhase::Moved,
            source: ScrollSource::Pointer,
        });
        vcx.run_until_parked();
    }

    /// BB-BUG-013: a long line's end lay past the pane with no way to reach it.
    #[gpui::test]
    fn end_scrolls_the_caret_into_sight_and_home_brings_the_text_back(cx: &mut TestAppContext) {
        let (rig, vcx): (Rig, &mut VisualTestContext) = long_line_rig(cx);
        let area: Bounds<Pixels> = vcx.debug_bounds("editor-area").unwrap();
        assert_eq!(scroll_x(&rig, vcx), 0.0);
        vcx.simulate_keystrokes("cmd-right");
        vcx.run_until_parked();
        let scrolled: f32 = scroll_x(&rig, vcx);
        assert!(
            scrolled > 0.0,
            "the line end lies past the pane: {scrolled}"
        );
        let caret: Bounds<Pixels> = vcx
            .debug_bounds("editor-caret")
            .expect("the caret is drawn");
        assert!(
            caret.left() >= area.left() && caret.right() <= area.right(),
            "caret {caret:?} within {area:?}"
        );
        let at_end: usize = cursor(&rig, vcx);
        vcx.simulate_click(caret.center(), Modifiers::default());
        assert_eq!(
            cursor(&rig, vcx),
            at_end,
            "a click where the caret is keeps the cursor"
        );
        vcx.simulate_keystrokes("cmd-left");
        vcx.run_until_parked();
        assert_eq!(scroll_x(&rig, vcx), 0.0, "Home brings the text back");
    }

    /// A trackpad swipe scrolls the text, never past the longest line nor
    /// left of it, and the caret stays where it was.
    #[gpui::test]
    fn a_swipe_scrolls_within_the_text(cx: &mut TestAppContext) {
        let (rig, vcx): (Rig, &mut VisualTestContext) = long_line_rig(cx);
        let area: Bounds<Pixels> = vcx.debug_bounds("editor-area").unwrap();
        let before: usize = cursor(&rig, vcx);
        swipe(vcx, area, -150.0);
        assert_eq!(scroll_x(&rig, vcx), 150.0, "a swipe scrolls");
        assert_eq!(cursor(&rig, vcx), before, "the caret stays put");
        swipe(vcx, area, -100_000.0);
        let limit: f32 = scroll_x(&rig, vcx);
        assert!(
            limit > 150.0 && limit < 10_000.0,
            "never past the longest line: {limit}"
        );
        swipe(vcx, area, 100_000.0);
        assert_eq!(scroll_x(&rig, vcx), 0.0, "and never left of the text");
    }
}
