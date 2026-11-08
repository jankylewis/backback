//! The text runs of one line: exactly the file's chars, coloured.

use super::super::super::bb_tabs::Tab;
use crate::intel::view::marks::{self, Ink, Mark};
use bb_syntax::{HighlightKind, HighlightSpan};
use std::time::Instant;

pub(crate) struct Run {
    pub(crate) text: String,
    pub(crate) kind: HighlightKind,
    pub(crate) selected: bool,
    /// A diagnostic squiggle under these chars.
    pub(crate) mark: Option<Ink>,
    /// Cmd + hover: these chars are a confirmed link (BBEOD15).
    pub(crate) link: bool,
}

#[cfg(test)]
pub(crate) fn styled_line(
    tab: &Tab,
    index: usize,
    marks: &[Mark],
    link: Option<(bb_text::CharIdx, bb_text::CharIdx)>,
) -> Vec<Run> {
    styled_part(tab, index, 0..usize::MAX, marks, link)
}

/// The runs of line `index`, only its chars in `columns` (a wrapped row).
pub(crate) fn styled_part(
    tab: &Tab,
    index: usize,
    columns: std::ops::Range<usize>,
    marks: &[Mark],
    link: Option<(bb_text::CharIdx, bb_text::CharIdx)>,
) -> Vec<Run> {
    let started: Instant = Instant::now();
    let text: String = tab.document.line(index).unwrap_or_default();
    let (clean, cut): (&str, bool) = super::caret::rendered(text.trim_end_matches(['\r', '\n']));
    let byte_start: usize = tab.document.line_start_byte(index).unwrap_or(0);
    let spans: Vec<HighlightSpan> = spans_in(tab, clean, byte_start, &columns);
    let char_start: usize = tab
        .document
        .line_start(index)
        .map_or(0, bb_text::CharIdx::raw);
    let selection: Option<(bb_text::CharIdx, bb_text::CharIdx)> = tab.document.selection();
    let mut runs: Vec<Run> = Vec::new();
    let skip: usize = columns.start;
    for (column, (byte, ch)) in clean
        .char_indices()
        .enumerate()
        .skip(skip)
        .take(columns.len())
    {
        let absolute: usize = byte_start.saturating_add(byte);
        let kind: HighlightKind = spans
            .iter()
            .rev()
            .find(|span: &&HighlightSpan| span.start <= absolute && absolute < span.end)
            .map_or(HighlightKind::Plain, |span: &HighlightSpan| span.kind);
        let at: usize = char_start.saturating_add(column);
        let selected: bool =
            selection.is_some_and(|(from, to): (bb_text::CharIdx, bb_text::CharIdx)| {
                from.raw() <= at && at < to.raw()
            });
        let linked: bool = link.is_some_and(|(from, to): (bb_text::CharIdx, bb_text::CharIdx)| {
            from.raw() <= at && at < to.raw()
        });
        let look: Look = (kind, selected, marks::at(marks, column), linked);
        push_run(&mut runs, ch, look);
    }
    if cut && (columns.end == usize::MAX || columns.end >= clean.chars().count()) {
        push_run(&mut runs, '…', (HighlightKind::Comment, false, None, false));
    }
    crate::latency::record_highlight(started.elapsed());
    runs
}

/// The highlight spans over chars `columns` of the line `clean`.
fn spans_in(
    tab: &Tab,
    clean: &str,
    byte_start: usize,
    columns: &std::ops::Range<usize>,
) -> Vec<HighlightSpan> {
    tab.highlighter.highlight_spans(
        tab.live.tree(),
        tab.document.rope(),
        byte_start.saturating_add(byte_of(clean, columns.start))
            ..byte_start.saturating_add(byte_of(clean, columns.end)),
    )
}

/// The byte where char `column` of `clean` starts; its end past the last.
/// Chars never outnumber bytes, so a column past the byte count is the end.
fn byte_of(clean: &str, column: usize) -> usize {
    if column >= clean.len() {
        return clean.len();
    }
    clean
        .char_indices()
        .nth(column)
        .map_or(clean.len(), |(byte, _): (usize, char)| byte)
}

type Look = (HighlightKind, bool, Option<Ink>, bool);

fn push_run(runs: &mut Vec<Run>, ch: char, (kind, selected, mark, link): Look) {
    if let Some(last) = runs.last_mut() {
        if last.kind == kind && last.selected == selected && last.mark == mark && last.link == link
        {
            last.text.push(ch);
            return;
        }
    }
    runs.push(Run {
        text: ch.to_string(),
        kind,
        selected,
        mark,
        link,
    });
}
