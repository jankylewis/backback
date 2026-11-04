//! The panel under the editor: rename preview, references, Find in Files
//! or Problems. A row click goes to its place. Its top edge drags its
//! height; rows keep their full width and the list scrolls both ways, the
//! match first and its path after it (BB-BUG-044).

use std::path::Path;

use bb_intel::{Diagnostic, DocPath, DocUri, FileChange, Location, Severity, TextChange};
use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, AnyElement, ClickEvent, Context, IntoElement, Window};

use crate::intel::bb_worker::protocol::Place;
use crate::intel::glue::jump;
use crate::intel::model::popups::RenamePreview;
use crate::{color, ExplorerView};

/// The panel to show, if any: the Debug panel while a session lives, then a
/// rename preview, references, Problems.
pub(crate) fn bottom(view: &ExplorerView, cx: &mut Context<ExplorerView>) -> Option<AnyElement> {
    if let Some(debug) = crate::run::bb_debug::view::panel(view, cx) {
        return Some(debug);
    }
    let theme: Theme = view.theme;
    let (title, rows): (String, Vec<AnyElement>) = if let Some(plan) = &view.intel.rename_preview {
        rename_rows(plan, &view.root, theme)
    } else if let Some(panel) = &view.intel.references {
        let rows: Vec<AnyElement> = panel
            .places
            .iter()
            .enumerate()
            .map(|(i, place): (usize, &Place)| place_row(i, place.clone(), &view.root, theme, cx))
            .collect();
        (panel.title.clone(), rows)
    } else if view.intel.problems_open {
        problem_rows(view, theme, cx)
    } else {
        return None;
    };
    Some(
        div()
            .id("intel-panel")
            .debug_selector(|| "intel-panel".to_owned())
            .h(px(view.lines.bottom_px))
            .flex_none()
            .flex()
            .flex_col()
            .bg(color(theme.surface))
            .text_color(color(theme.text))
            .child(crate::editor::bottom_handle(theme, cx))
            .child(
                super::one_line(22.0)
                    .px_2()
                    .text_color(color(theme.text_dim))
                    .child(title),
            )
            .child(row_list(view, rows))
            .into_any_element(),
    )
}

/// The rows, scrolled both ways: a row keeps its full width (BB-BUG-044).
fn row_list(view: &ExplorerView, rows: Vec<AnyElement>) -> impl IntoElement {
    div()
        .id("intel-rows")
        .debug_selector(|| "intel-rows".to_owned())
        .flex_1()
        .min_h(px(0.0))
        .overflow_scroll()
        .track_scroll(&view.intel.rows_scroll)
        .flex()
        .flex_col()
        .items_start()
        .children(rows)
}

fn short(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn place_row(
    index: usize,
    place: Place,
    root: &Path,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    let name: String = match &place.location.uri {
        DocUri::File(path) => short(path.as_path(), root),
        DocUri::Virtual(uri) => uri.clone(),
    };
    let line: u32 = place.location.span.start.line.saturating_add(1);
    let column: u32 = place.location.span.start.character.saturating_add(1);
    let place_text: String = format!("{name}:{line}:{column}");
    div()
        .id(("place", index))
        .debug_selector(move || format!("place-{index}"))
        .min_h(px(20.0))
        .min_w_full()
        .flex_none()
        .flex()
        .items_center()
        .whitespace_nowrap()
        .px_2()
        .hover(|el: gpui::StyleRefinement| el.bg(color(theme.overlay)))
        .child(place.preview.trim().to_owned())
        .child(
            div()
                .debug_selector(move || format!("place-path-{index}"))
                .ml_4()
                .text_color(color(theme.text_dim))
                .child(place_text),
        )
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                jump(view, &place, window, cx);
                cx.notify();
            },
        ))
        .into_any_element()
}

fn problem_rows(
    view: &ExplorerView,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> (String, Vec<AnyElement>) {
    let problems: Vec<(&Path, &Diagnostic)> = view.intel.problems_in(&view.root);
    let title: String = format!("Problems ({})", problems.len());
    let rows: Vec<AnyElement> = problems
        .into_iter()
        .enumerate()
        .filter_map(|(i, (path, d)): (usize, (&Path, &Diagnostic))| {
            let place: Place = Place {
                location: Location {
                    uri: DocUri::File(DocPath::new(path.to_path_buf()).ok()?),
                    span: d.span,
                },
                preview: format!(
                    "{} {} {}",
                    severity_word(d.severity),
                    d.code.clone().unwrap_or_default(),
                    d.message
                ),
            };
            Some(place_row(i, place, &view.root, theme, cx))
        })
        .collect();
    (title, rows)
}

fn severity_word(severity: Option<Severity>) -> &'static str {
    match severity {
        Some(Severity::Error) => "error",
        Some(Severity::Warning) => "warning",
        Some(Severity::Information) => "info",
        Some(Severity::Hint) => "hint",
        None => "note",
    }
}

fn rename_rows(plan: &RenamePreview, root: &Path, theme: Theme) -> (String, Vec<AnyElement>) {
    let (files, edits): (usize, usize) = plan.counts();
    let title: String = format!(
        "Rename to '{}': {edits} edits in {files} files (Enter: apply, Esc: cancel)",
        plan.name
    );
    let rows: Vec<AnyElement> = plan
        .changes
        .iter()
        .map(|change: &FileChange| {
            let text: String = match change {
                FileChange::Edit { uri, changes } => {
                    let lines: Vec<String> = changes
                        .iter()
                        .map(|c: &TextChange| c.span.start.line.saturating_add(1).to_string())
                        .collect();
                    format!("{}  lines {}", uri_name(uri, root), lines.join(", "))
                }
                FileChange::Create(uri) => format!("create {}", uri_name(uri, root)),
                FileChange::Rename { from, to } => {
                    format!("move {} -> {}", uri_name(from, root), uri_name(to, root))
                }
                FileChange::Delete(uri) => format!("delete {}", uri_name(uri, root)),
            };
            super::one_line(20.0)
                .px_2()
                .text_color(color(theme.text))
                .child(text)
                .into_any_element()
        })
        .collect();
    (title, rows)
}

fn uri_name(uri: &DocUri, root: &Path) -> String {
    match uri {
        DocUri::File(path) => short(path.as_path(), root),
        DocUri::Virtual(name) => name.clone(),
    }
}
