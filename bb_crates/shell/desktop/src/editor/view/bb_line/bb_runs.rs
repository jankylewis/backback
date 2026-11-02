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

pub(crate) fn styled_line(
    tab: &Tab,
    index: usize,
    marks: &[Mark],
    link: Option<(bb_text::CharIdx, bb_text::CharIdx)>,
) -> Vec<Run> {
    let started: Instant = Instant::now();
    let text: String = tab.document.line(index).unwrap_or_default();
    let clean: &str = text.trim_end_matches(['\r', '\n']);
    let byte_start: usize = tab.document.line_start_byte(index).unwrap_or(0);
    let spans: Vec<HighlightSpan> = tab.highlighter.highlight_spans(
        tab.live.tree(),
        tab.document.rope(),
        byte_start..byte_start.saturating_add(clean.len()),
    );
    let char_start: usize = tab
        .document
        .line_start(index)
        .map_or(0, bb_text::CharIdx::raw);
    let selection: Option<(bb_text::CharIdx, bb_text::CharIdx)> = tab.document.selection();
    let mut runs: Vec<Run> = Vec::new();
    for (column, (byte, ch)) in clean.char_indices().enumerate() {
        let absolute: usize = byte_start.saturating_add(byte);
        let kind: HighlightKind = spans
            .iter()
            .rev()
            .find(|span: &&HighlightSpan| span.start <= absolute && absolute < span.end)
            .map_or(HighlightKind::Plain, |span: &HighlightSpan| span.kind);
        let selected: bool =
            selection.is_some_and(|(from, to): (bb_text::CharIdx, bb_text::CharIdx)| {
                from.raw() <= char_start.saturating_add(column)
                    && char_start.saturating_add(column) < to.raw()
            });
        let at: usize = char_start.saturating_add(column);
        let linked: bool = link.is_some_and(|(from, to): (bb_text::CharIdx, bb_text::CharIdx)| {
            from.raw() <= at && at < to.raw()
        });
        let look: Look = (kind, selected, marks::at(marks, column), linked);
        push_run(&mut runs, ch, look);
    }
    crate::latency::record_highlight(started.elapsed());
    runs
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
