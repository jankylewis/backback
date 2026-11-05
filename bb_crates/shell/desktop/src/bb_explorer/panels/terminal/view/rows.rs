//! The terminal's rows as drawn: plain, selected (BB-BUG-053, in the
//! theme's `selection`) and cursor cells.

use gpui::{div, prelude::*, px, AnyElement};

use super::super::bb_session::Screen;
use crate::color;

/// A run of columns on one row: (first, last), both included.
type Span = (usize, usize);

/// Every screen row, with its cursor and selected columns.
pub(super) fn drawn(screen: Screen, theme: bb_ui::Theme, height: f32) -> Vec<gpui::Div> {
    let (cursor, selected): (Option<(usize, usize)>, Vec<Option<Span>>) =
        (screen.cursor, screen.selected);
    screen
        .rows
        .into_iter()
        .enumerate()
        .map(|(index, text): (usize, String)| {
            let at: Option<usize> = cursor
                .filter(|(line, _): &(usize, usize)| *line == index)
                .map(|(_, column): (usize, usize)| column);
            let chosen: Option<(usize, usize)> = selected.get(index).copied().flatten();
            row(&text, (at, chosen), theme, height)
        })
        .collect()
}

/// How a cell is drawn.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Look {
    Plain,
    Selected,
    Cursor,
}

/// One screen row: runs of plain, selected and cursor cells.
fn row(
    text: &str,
    (cursor, selected): (Option<usize>, Option<(usize, usize)>),
    theme: bb_ui::Theme,
    height: f32,
) -> gpui::Div {
    let mut chars: Vec<char> = text.chars().collect();
    let need: usize = cursor
        .map_or(0, |c: usize| c.saturating_add(1))
        .max(selected.map_or(0, |(_, last): (usize, usize)| last.saturating_add(1)));
    if chars.len() < need {
        chars.resize(need, ' ');
    }
    let mut out: gpui::Div = div().h(px(height)).flex().flex_none().whitespace_nowrap();
    let (mut run, mut now): (String, Look) = (String::new(), Look::Plain);
    for (i, c) in chars.into_iter().enumerate() {
        let next: Look = look(i, cursor, selected);
        if next != now && !run.is_empty() {
            out = out.child(piece(std::mem::take(&mut run), now, theme));
        }
        now = next;
        run.push(c);
    }
    if run.is_empty() {
        return out;
    }
    out.child(piece(run, now, theme))
}

fn look(i: usize, cursor: Option<usize>, selected: Option<(usize, usize)>) -> Look {
    if cursor == Some(i) {
        Look::Cursor
    } else if selected.is_some_and(|(a, b): (usize, usize)| a <= i && i <= b) {
        Look::Selected
    } else {
        Look::Plain
    }
}

fn piece(text: String, look: Look, theme: bb_ui::Theme) -> AnyElement {
    match look {
        Look::Plain => text.into_any_element(),
        Look::Selected => div()
            .debug_selector(|| "terminal-selected".to_owned())
            .bg(color(theme.selection))
            .text_color(color(theme.text))
            .child(text)
            .into_any_element(),
        Look::Cursor => div()
            .debug_selector(|| "terminal-cursor".to_owned())
            .bg(color(theme.text))
            .text_color(color(theme.bg))
            .child(text)
            .into_any_element(),
    }
}
