use super::super::bb_tabs::Tab;
use crate::{color, ExplorerView};
use bb_syntax::{HighlightKind, HighlightSpan};
use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, Context, IntoElement, Window};
use std::time::Instant;

pub(super) struct Run {
    text: String,
    kind: HighlightKind,
    selected: bool,
}

pub(super) fn styled_line(tab: &Tab, index: usize) -> Vec<Run> {
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
    let cursor: Option<usize> = (tab.document.cursor_line() == index)
        .then(|| tab.document.cursor().raw().saturating_sub(char_start));
    let mut runs: Vec<Run> = Vec::new();
    for (column, (byte, ch)) in clean.char_indices().enumerate() {
        if cursor == Some(column) {
            push_run(&mut runs, '│', HighlightKind::Plain, false);
        }
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
        push_run(&mut runs, ch, kind, selected);
    }
    if cursor == Some(clean.chars().count()) {
        push_run(&mut runs, '│', HighlightKind::Plain, false);
    }
    crate::latency::record_highlight(started.elapsed());
    runs
}

fn push_run(runs: &mut Vec<Run>, ch: char, kind: HighlightKind, selected: bool) {
    if let Some(last) = runs.last_mut() {
        if last.kind == kind && last.selected == selected {
            last.text.push(ch);
            return;
        }
    }
    runs.push(Run {
        text: ch.to_string(),
        kind,
        selected,
    });
}

pub(super) fn line_row(
    index: usize,
    runs: Vec<Run>,
    current: bool,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    div()
        .id(index)
        .h(px(22.0))
        .flex()
        .items_center()
        .font_family("Menlo")
        .text_size(px(13.0))
        .text_color(color(theme.text))
        .bg(color(if current { theme.surface } else { theme.bg }))
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(
                move |view: &mut ExplorerView,
                      event: &gpui::MouseDownEvent,
                      window: &mut Window,
                      cx: &mut Context<ExplorerView>| {
                    mouse_down(index, view, event, window, cx);
                },
            ),
        )
        .on_mouse_move(cx.listener(
            move |view: &mut ExplorerView,
                  event: &gpui::MouseMoveEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                mouse_move(index, view, event, window, cx);
            },
        ))
        .child(
            div()
                .w(px(50.0))
                .text_color(color(theme.text_dim))
                .child(format!("{:>4} ", index.saturating_add(1))),
        )
        .children(
            runs.into_iter()
                .map(move |run: Run| run_segment(run, current, theme)),
        )
}

fn run_segment(run: Run, current: bool, theme: Theme) -> impl IntoElement {
    let ink: bb_ui::Rgb = match run.kind {
        HighlightKind::Keyword => theme.keyword,
        HighlightKind::String => theme.string,
        HighlightKind::Comment => theme.comment,
        HighlightKind::Type => theme.ty,
        HighlightKind::Plain => theme.text,
    };
    div()
        .text_color(color(ink))
        .bg(color(if run.selected {
            theme.accent
        } else if current {
            theme.surface
        } else {
            theme.bg
        }))
        .child(run.text)
}

fn mouse_down(
    index: usize,
    view: &mut ExplorerView,
    event: &gpui::MouseDownEvent,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    super::super::input::pointer_at(
        view,
        index,
        f32::from(event.position.x),
        event.modifiers.shift,
        window,
        cx,
    );
}

fn mouse_move(
    index: usize,
    view: &mut ExplorerView,
    event: &gpui::MouseMoveEvent,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    if event.dragging() {
        super::super::input::pointer_at(view, index, f32::from(event.position.x), true, window, cx);
    }
}
