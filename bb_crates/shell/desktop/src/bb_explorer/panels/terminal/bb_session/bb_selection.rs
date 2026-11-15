//! The mouse selection (BB-BUG-053): alacritty's own, in grid points, so
//! it moves with the text as the shell scrolls.

use alacritty_terminal::index::{Column, Line, Point, Side};
use alacritty_terminal::selection::{Selection, SelectionRange, SelectionType};
use alacritty_terminal::term::Term;

use super::{Cell, Proxy, Session, Size};

impl Session {
    /// A mouse press: a new selection starts at `cell` (an old one goes).
    pub(crate) fn select_from(&self, cell: Cell) {
        let term: &mut Term<Proxy> = &mut self.term.lock();
        let (point, side): (Point, Side) = grid_point(term, cell);
        term.selection = Some(Selection::new(SelectionType::Simple, point, side));
    }

    /// A drag: the selection now ends at `cell`.
    pub(crate) fn select_to(&self, cell: Cell) {
        let term: &mut Term<Proxy> = &mut self.term.lock();
        let (point, side): (Point, Side) = grid_point(term, cell);
        if let Some(selection) = term.selection.as_mut() {
            selection.update(point, side);
        }
    }

    /// The selected text, lines joined by `\n`; `None` when nothing is.
    pub(crate) fn selected_text(&self) -> Option<String> {
        self.term
            .lock()
            .selection_to_string()
            .filter(|text: &String| !text.is_empty())
    }

    /// Drop the selection (a key typed, a click without a drag).
    pub(crate) fn clear_selection(&self) {
        self.term.lock().selection = None;
    }
}

/// The grid point of a screen cell, under the current scroll.
fn grid_point(term: &Term<Proxy>, cell: Cell) -> (Point, Side) {
    let offset: i32 = i32::try_from(term.grid().display_offset()).unwrap_or(0);
    let row: i32 = i32::try_from(cell.row).unwrap_or(i32::MAX);
    let side: Side = if cell.right_half {
        Side::Right
    } else {
        Side::Left
    };
    (
        Point::new(Line(row.saturating_sub(offset)), Column(cell.column)),
        side,
    )
}

/// Per screen row, the columns `range` covers (first, last).
pub(super) fn selected_rows(
    range: Option<SelectionRange>,
    offset: i32,
    size: Size,
) -> Vec<Option<(usize, usize)>> {
    let last: usize = size.columns.saturating_sub(1);
    (0..size.lines)
        .map(|row: usize| {
            let range: SelectionRange = range?;
            let line: i32 = i32::try_from(row).ok()?.checked_sub(offset)?;
            if line < range.start.line.0 || line > range.end.line.0 {
                return None;
            }
            let first: usize = if line == range.start.line.0 {
                range.start.column.0
            } else {
                0
            };
            let end: usize = if line == range.end.line.0 {
                range.end.column.0
            } else {
                last
            };
            Some((first, end.min(last)))
        })
        .collect()
}
