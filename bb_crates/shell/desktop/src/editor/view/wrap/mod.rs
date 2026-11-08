//! Word wrap (Settings > Word wrap, Alt+Z; off by default): each pane's
//! file shows as screen rows that fit the pane, so a long line never needs
//! a sideways scroll. Rows are cached per tab and rebuilt only when the
//! text, the pane width or the font changes.

mod bb_rows;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod window_tests;

use std::collections::HashMap;

use gpui::{px, App, Global, SharedString, TextRun, Window};

use super::super::bb_tabs::Tab;
use super::super::{Group, Tabs};
use crate::ExplorerView;
pub(crate) use bb_rows::{Row, Rows};

/// Files bigger than this many chars do not wrap (they scroll sideways).
const MAX_CHARS: usize = 4_000_000;

/// A tab's rows and what they were built for.
pub(crate) struct Wrapped {
    key: Key,
    pub(crate) rows: Rows,
}

#[derive(Debug, PartialEq)]
struct Key {
    revision: u64,
    width: u32,
    family: String,
    size: u32,
}

/// Each char's advance in the editor font, measured once per font.
#[derive(Default)]
struct Advances {
    font: (String, u32),
    widths: HashMap<char, f32>,
}

impl Global for Advances {}

/// Before the panes draw: (re)build the rows of each shown tab while word
/// wrap is on; drop them while it is off.
pub(crate) fn refresh(view: &mut ExplorerView, window: &Window, cx: &mut App) {
    let fonts: bb_ui::Typography = crate::settings::typography(cx);
    let font: (String, u32) = (fonts.family.clone(), fonts.size().to_bits());
    let advances: &mut Advances = cx.default_global::<Advances>();
    if advances.font != font {
        *advances = Advances {
            font: font.clone(),
            widths: HashMap::new(),
        };
    }
    for g in 0..view.tabs.groups() {
        let width: f32 = text_width(view, g, &fonts);
        let Some(tab): Option<&mut Tab> = view.tabs.active_tab_mut_of(Group(g)) else {
            continue;
        };
        if !fonts.wrap || tab.document.len_chars() > MAX_CHARS {
            tab.wrap = None;
            continue;
        }
        let key: Key = Key {
            revision: tab.document.revision(),
            width: width.to_bits(),
            family: font.0.clone(),
            size: font.1,
        };
        if tab.wrap.as_ref().is_some_and(|w: &Wrapped| w.key == key) {
            continue;
        }
        let advance: &mut dyn FnMut(char) -> f32 =
            &mut |c: char| -> f32 { measure(advances, c, window, &fonts) };
        let document: &bb_text::Document = &tab.document;
        let rows: Rows = Rows::build(
            (0..document.len_lines()).map(|line: usize| -> String {
                let text: String = document.line(line).unwrap_or_default();
                let clean: &str = text.trim_end_matches(['\r', '\n']);
                super::bb_line::rendered(clean).0.to_owned()
            }),
            width,
            advance,
        );
        // Every row fits: nothing scrolls sideways while wrapping.
        (tab.scroll_x, tab.widest_px) = (0.0, 0.0);
        tab.wrap = Some(Wrapped { key, rows });
    }
}

/// The px a row of pane `g` may fill: the pane less the gutter, the bar
/// and room for the caret.
fn text_width(view: &ExplorerView, g: usize, fonts: &bb_ui::Typography) -> f32 {
    let pane: f32 = view.lines.widths.get(g).copied().unwrap_or(0.0);
    let reserved: f32 =
        fonts.gutter_width() + super::bb_scrollbar::BAR_PX + super::bb_line::CARET_W;
    (pane - reserved - 4.0).max(1.0)
}

fn measure(advances: &mut Advances, c: char, window: &Window, fonts: &bb_ui::Typography) -> f32 {
    if let Some(width) = advances.widths.get(&c) {
        return *width;
    }
    let text: String = c.to_string();
    let run: TextRun = TextRun {
        len: text.len(),
        font: gpui::font(fonts.family.clone()),
        color: gpui::black(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let width: f32 = f32::from(
        window
            .text_system()
            .shape_line(SharedString::from(text), px(fonts.size()), &[run], None)
            .width,
    );
    advances.widths.insert(c, width);
    width
}

/// Screen rows of `tab`: one per line unless it wraps.
pub(crate) fn row_count(tab: &Tab) -> usize {
    tab.wrap
        .as_ref()
        .map_or_else(|| tab.document.len_lines(), |w: &Wrapped| w.rows.len())
}

/// Screen row `index` of `tab`, when it wraps.
pub(crate) fn row(tab: &Tab, index: usize) -> Option<Row> {
    tab.wrap.as_ref().and_then(|w: &Wrapped| w.rows.get(index))
}

/// The file lines rows `range` show.
pub(crate) fn lines_of(tab: &Tab, range: &std::ops::Range<usize>) -> std::ops::Range<usize> {
    tab.wrap
        .as_ref()
        .map_or_else(|| range.clone(), |w: &Wrapped| w.rows.lines_of(range))
}

/// The row that shows `line`: the cursor's row when the cursor is on it.
pub(crate) fn row_at(tab: &Tab, line: usize) -> usize {
    let Some(wrapped): Option<&Wrapped> = tab.wrap.as_ref() else {
        return line;
    };
    let column: usize = if tab.document.cursor_line() == line {
        let start: usize = tab
            .document
            .line_start(line)
            .map_or(0, bb_text::CharIdx::raw);
        tab.document.cursor().raw().saturating_sub(start)
    } else {
        0
    };
    wrapped.rows.row_of(line, column)
}

/// Scroll the focused pane to show `line` of its file.
pub(crate) fn reveal(tabs: &Tabs, line: usize, strategy: gpui::ScrollStrategy) {
    let row: usize = tabs
        .active_tab()
        .map_or(line, |tab: &Tab| row_at(tab, line));
    tabs.scroll().scroll_to_item(row, strategy);
}

/// What screen row `index` of `tab` shows: a whole line, or a wrapped part.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Part {
    pub(crate) line: usize,
    pub(crate) columns: std::ops::Range<usize>,
    pub(crate) indent: f32,
    pub(crate) first: bool,
    pub(crate) last: bool,
}

/// Row `index` of `tab`: its wrapped part, or line `index` whole.
pub(crate) fn part(tab: &Tab, index: usize) -> Part {
    row(tab, index).map_or(
        Part {
            line: index,
            columns: 0..usize::MAX,
            indent: 0.0,
            first: true,
            last: true,
        },
        |r: Row| Part {
            line: r.line,
            columns: r.start..r.end,
            indent: r.indent,
            first: r.first,
            last: r.last,
        },
    )
}

/// The chars `part` shows (no line break).
pub(crate) fn part_text(tab: &Tab, part: &Part) -> String {
    let text: String = tab.document.line(part.line).unwrap_or_default();
    let clean: &str = text.trim_end_matches(['\r', '\n']);
    if part.columns.start == 0 && part.columns.end == usize::MAX {
        return clean.to_owned();
    }
    clean
        .chars()
        .skip(part.columns.start)
        .take(part.columns.len())
        .collect()
}
