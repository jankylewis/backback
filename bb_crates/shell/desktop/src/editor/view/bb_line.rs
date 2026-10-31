use super::super::bb_tabs::Tab;
use crate::{color, ExplorerView};
use bb_syntax::{HighlightKind, HighlightSpan};
use bb_ui::bb_theme::DARK;
use gpui::{div, prelude::*, px, Context, IntoElement, Window};

pub(super) struct Run {
    text: String,
    kind: HighlightKind,
    selected: bool,
}

pub(super) fn styled_line(tab: &Tab, index: usize) -> Vec<Run> {
    let text: String = tab.document.line(index).unwrap_or_default();
    let clean: &str = text.trim_end_matches(['\r', '\n']);
    let byte_start: usize = tab.line_bytes.get(index).copied().unwrap_or(0);
    let spans: Vec<HighlightSpan> = tab.syntax.highlight_spans(
        &tab.tree,
        &tab.parsed_text,
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
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    div()
        .id(index)
        .h(px(22.0))
        .flex()
        .items_center()
        .font_family("Menlo")
        .text_size(px(13.0))
        .text_color(color(DARK.text))
        .bg(color(if current { DARK.surface } else { DARK.bg }))
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
                .text_color(color(DARK.text_dim))
                .child(format!("{:>4} ", index.saturating_add(1))),
        )
        .children(
            runs.into_iter()
                .map(move |run: Run| run_segment(run, current)),
        )
}

fn run_segment(run: Run, current: bool) -> impl IntoElement {
    let ink: bb_ui::Rgb = match run.kind {
        HighlightKind::Keyword => DARK.keyword,
        HighlightKind::String => DARK.string,
        HighlightKind::Comment => DARK.comment,
        HighlightKind::Type => DARK.ty,
        HighlightKind::Plain => DARK.text,
    };
    div()
        .text_color(color(ink))
        .bg(color(if run.selected {
            DARK.accent
        } else if current {
            DARK.surface
        } else {
            DARK.bg
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
