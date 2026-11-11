//! Bracket matching, as Rider shows it: with the cursor next to a bracket,
//! both brackets of the pair get a box, and a `{ }` pair over several lines
//! gets a thin guide between them. The scan counts brackets as written (it
//! does not skip strings or comments) and gives up after [`SCAN_CAP`] chars.

use bb_text::Document;
use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, Window};

use super::super::bb_line::Columns;
use super::super::wrap::{self, Part};
use crate::color;
use crate::editor::Tab;

/// Chars looked at for the other bracket before giving up.
const SCAN_CAP: usize = 200_000;
const PAIRS: [(char, char); 3] = [('{', '}'), ('(', ')'), ('[', ']')];

/// A matched pair: each bracket's (line, column), and whether it is `{ }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Pair {
    pub(crate) open: (usize, usize),
    pub(crate) close: (usize, usize),
    pub(crate) curly: bool,
}

/// The pair of the bracket right after the cursor, else right before it.
pub(crate) fn pair(document: &Document) -> Option<Pair> {
    let cursor: usize = document.cursor().raw();
    matched(document, cursor).or_else(|| matched(document, cursor.checked_sub(1)?))
}

fn matched(document: &Document, at: usize) -> Option<Pair> {
    let ch: char = document.rope().get_char(at)?;
    let (open, close): (char, char) = PAIRS
        .into_iter()
        .find(|(open, close): &(char, char)| ch == *open || ch == *close)?;
    let forward: bool = ch == open;
    let mut depth: usize = 0;
    let mut index: usize = at;
    for _ in 0..SCAN_CAP {
        let seen: char = document.rope().get_char(index)?;
        if seen == ch {
            depth = depth.saturating_add(1);
        } else if seen == if forward { close } else { open } {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                let (first, second): (usize, usize) = (at.min(index), at.max(index));
                return Some(Pair {
                    open: place(document, first)?,
                    close: place(document, second)?,
                    curly: open == '{',
                });
            }
        }
        index = if forward {
            index.checked_add(1)?
        } else {
            index.checked_sub(1)?
        };
    }
    None
}

fn place(document: &Document, at: usize) -> Option<(usize, usize)> {
    let line: usize = document.rope().try_char_to_line(at).ok()?;
    Some((line, at.checked_sub(document.line_start(line)?.raw())?))
}

/// The pair at `tab`'s cursor, found once per cursor place and text revision.
fn cached(tab: &Tab) -> Option<Pair> {
    let key: (u64, usize) = (tab.document.revision(), tab.document.cursor().raw());
    match tab.braces.get() {
        Some((seen, found)) if seen == key => found,
        Some(_) | None => {
            let found: Option<Pair> = pair(&tab.document);
            tab.braces.set(Some((key, found)));
            found
        }
    }
}

/// The boxes and the guide row `part` draws, placed from the row's left edge.
pub(crate) fn quads(
    tab: &Tab,
    part: &Part,
    (scroll_x, theme): (f32, Theme),
    window: &Window,
    fonts: &bb_ui::Typography,
) -> Vec<gpui::Div> {
    let Some(found): Option<Pair> = cached(tab) else {
        return Vec::new();
    };
    let line: usize = part.line;
    let mut out: Vec<gpui::Div> = Vec::new();
    let edge: (f32, f32) = (scroll_x, fonts.gutter_width());
    for (at, column) in [found.open, found.close] {
        if at != line || !part.columns.contains(&column) {
            continue;
        }
        let shaped: Columns = Columns::shape(window, &wrap::part_text(tab, part), fonts);
        let inside: usize = column.saturating_sub(part.columns.start);
        let (from, to): (f32, f32) = (shaped.x_of(inside), shaped.x_of(inside.saturating_add(1)));
        if let Some(x) = left(part.indent + from, edge) {
            let height: f32 = fonts.size() + 3.0;
            out.push(
                quad(x, to - from, theme)
                    .top(px((fonts.line_height() - height) / 2.0))
                    .h(px(height))
                    .opacity(0.4)
                    .debug_selector(move || format!("brace-box-{line}")),
            );
        }
    }
    let between: bool = found.open.0 < line && line < found.close.0;
    if found.curly && between && tab.wrap.is_none() {
        let column: usize = found.open.1.min(found.close.1);
        let indent: f32 = Columns::shape(window, &" ".repeat(column), fonts).x_of(column);
        if let Some(x) = left(indent, edge) {
            out.push(
                quad(x, 1.0, theme)
                    .top_0()
                    .h_full()
                    .debug_selector(move || format!("brace-guide-{line}")),
            );
        }
    }
    out
}

/// `x` from the text's left edge, as an x in the row; `None` when it has
/// scrolled under the gutter.
fn left(x: f32, (scroll_x, gutter): (f32, f32)) -> Option<f32> {
    let shown: f32 = x - scroll_x;
    (shown >= 0.0).then_some(gutter + shown)
}

fn quad(x: f32, width: f32, theme: Theme) -> gpui::Div {
    div()
        .absolute()
        .left(px(x))
        .w(px(width))
        .bg(color(theme.warn))
}

#[cfg(test)]
mod tests {
    use bb_text::{CharIdx, Document};
    use gpui::{Bounds, Pixels, TestAppContext, VisualTestContext};

    use super::{pair, Pair};
    use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
    use crate::ExplorerView;

    const TEXT: &str = "class A\n{\n    int F(int[] a) { return a[0]; }\n}\n";

    type Found = Option<((usize, usize), (usize, usize), bool)>;

    fn at(cursor: usize) -> Found {
        let mut document: Document = Document::from_text(TEXT, None);
        document.set_cursor(CharIdx::new(cursor)).unwrap();
        pair(&document).map(|p: Pair| (p.open, p.close, p.curly))
    }

    #[test]
    fn the_bracket_at_the_cursor_finds_its_partner_either_way() {
        let outer: Found = Some(((1, 0), (3, 0), true));
        assert_eq!(at(8), outer, "before the opening brace");
        assert_eq!(at(9), outer, "after it");
        assert_eq!(at(46), outer, "before the closing brace");
        assert_eq!(at(3), None, "no bracket at the cursor");
        assert_eq!(at(19), Some(((2, 9), (2, 17), false)), "( to )");
        assert_eq!(at(24), Some(((2, 13), (2, 14), false)), "[ to ]");
        assert_eq!(at(29), Some(((2, 19), (2, 34), true)), "the inner braces");
    }

    #[gpui::test]
    fn both_braces_get_a_box_and_the_lines_between_a_guide(cx: &mut TestAppContext) {
        let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, TEXT);
        assert!(vcx.debug_bounds("brace-box-1").is_none(), "cursor at 0");
        rig.view.update(
            vcx,
            |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
                v.tabs
                    .active_mut()
                    .unwrap()
                    .set_cursor(CharIdx::new(8))
                    .unwrap();
                cx.notify();
            },
        );
        vcx.run_until_parked();
        let open: Bounds<Pixels> = vcx.debug_bounds("brace-box-1").unwrap();
        let close: Bounds<Pixels> = vcx.debug_bounds("brace-box-3").unwrap();
        let guide: Bounds<Pixels> = vcx.debug_bounds("brace-guide-2").unwrap();
        assert_eq!(open.left(), close.left());
        assert_eq!(guide.left(), open.left(), "the guide sits under the brace");
        assert!(vcx.debug_bounds("brace-guide-1").is_none());
        assert!(vcx.debug_bounds("brace-guide-3").is_none());
    }
}
