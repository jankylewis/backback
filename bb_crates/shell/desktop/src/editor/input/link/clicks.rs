//! A press in the text: one places the cursor (with Cmd it also asks for the
//! definition), two select the word under it, three the line's content
//! without its indent, as VS Code and Rider do.

use bb_text::{CharIdx, Document};
use gpui::{Context, Window};

use super::bb_link::word_at;
use crate::ExplorerView;

pub(crate) fn press(
    row: usize,
    view: &mut ExplorerView,
    event: &gpui::MouseDownEvent,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let x: f32 = f32::from(event.position.x);
    super::super::pointer_at(view, row, x, event.modifiers.shift, window, cx);
    if let Some(document) = view.tabs.active_mut() {
        if let Some((from, to)) = span(document, event.click_count) {
            if let Err(error) = document.select(from, to) {
                view.status = Some(error.to_string());
            }
        }
    }
    if event.modifiers.platform {
        view.intel.link = None;
        crate::intel::glue::ask_at_cursor(
            view,
            crate::intel::bb_worker::protocol::Ask::Definition,
            None,
        );
    }
}

/// What `clicks` presses at the cursor select: the word for two, the line's
/// content for three or more; nothing for one, or where there is no text.
pub(crate) fn span(document: &Document, clicks: usize) -> Option<(CharIdx, CharIdx)> {
    let cursor: CharIdx = document.cursor();
    match clicks {
        0 | 1 => None,
        // A press on the right half of a word's last char lands after it.
        2 => word_at(document, cursor).or_else(|| {
            let before: usize = cursor.raw().checked_sub(1)?;
            word_at(document, CharIdx::new(before))
        }),
        _ => {
            let line: usize = document.cursor_line();
            let start: usize = document.line_start(line)?.raw();
            let text: String = document.line(line)?;
            let content: &str = text.trim_end();
            let indent: usize = content
                .chars()
                .take_while(|c: &char| c.is_whitespace())
                .count();
            let end: usize = content.chars().count();
            (indent < end).then(|| {
                (
                    CharIdx::new(start.saturating_add(indent)),
                    CharIdx::new(start.saturating_add(end)),
                )
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use bb_text::{CharIdx, Document};
    use gpui::{point, px, Modifiers, MouseButton, TestAppContext, VisualTestContext};

    use super::span;
    use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
    use crate::ExplorerView;

    const TEXT: &str = "AdoSuite(63708)\n    [AdoSuite(63708)]  \n\n";

    fn at(cursor: usize, clicks: usize) -> Option<(usize, usize)> {
        let mut document: Document = Document::from_text(TEXT, None);
        document.set_cursor(CharIdx::new(cursor)).unwrap();
        span(&document, clicks).map(|(a, b): (CharIdx, CharIdx)| (a.raw(), b.raw()))
    }

    #[test]
    fn two_clicks_take_the_word_and_three_the_line_without_its_indent() {
        assert_eq!(at(3, 1), None);
        assert_eq!(at(3, 2), Some((0, 8)), "inside AdoSuite");
        assert_eq!(at(8, 2), Some((0, 8)), "just after its last char");
        assert_eq!(at(10, 2), Some((9, 14)), "the number");
        assert_eq!(at(16, 2), None, "in the indent");
        assert_eq!(at(18, 3), Some((20, 37)), "[AdoSuite(63708)]");
        assert_eq!(at(18, 4), Some((20, 37)), "more clicks stay on the line");
        assert_eq!(at(40, 3), None, "an empty line");
    }

    fn click(vcx: &mut VisualTestContext, line: &'static str, clicks: usize) {
        let text: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds(line).unwrap();
        let position: gpui::Point<gpui::Pixels> = point(text.left() + px(3.0), text.center().y);
        vcx.simulate_event(gpui::MouseDownEvent {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: clicks,
            first_mouse: false,
        });
        vcx.simulate_event(gpui::MouseUpEvent {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: clicks,
        });
        vcx.run_until_parked();
    }

    fn selected(rig: &Rig, vcx: &mut VisualTestContext) -> Option<(usize, usize)> {
        rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
            let range: Option<(CharIdx, CharIdx)> = v.tabs.active().unwrap().selection();
            range.map(|(a, b): (CharIdx, CharIdx)| (a.raw(), b.raw()))
        })
    }

    #[gpui::test]
    fn a_double_click_selects_the_word_and_a_triple_click_the_line(cx: &mut TestAppContext) {
        let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, TEXT);
        click(vcx, "editor-text-0", 1);
        assert_eq!(selected(&rig, vcx), None);
        click(vcx, "editor-text-0", 2);
        assert_eq!(selected(&rig, vcx), Some((0, 8)));
        click(vcx, "editor-text-1", 3);
        assert_eq!(selected(&rig, vcx), Some((20, 37)));
    }
}
