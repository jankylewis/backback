//! The panel under the editor: rename preview, references or Problems.
//! A row click goes to its place.

use std::path::Path;

use bb_intel::{Diagnostic, DocPath, DocUri, FileChange, Location, Severity, TextChange};
use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, AnyElement, ClickEvent, Context, IntoElement, Window};

use crate::intel::bb_worker::protocol::Place;
use crate::intel::glue::jump;
use crate::intel::model::popups::RenamePreview;
use crate::{color, ExplorerView};

/// The panel to show, if any: a rename preview first, then references,
/// then Problems.
pub(crate) fn bottom(view: &ExplorerView, cx: &mut Context<ExplorerView>) -> Option<AnyElement> {
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
            .h(px(180.0))
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .bg(color(theme.surface))
            .border_t_1()
            .border_color(color(theme.border))
            .font_family("Menlo")
            .text_size(px(12.0))
            .text_color(color(theme.text))
            .child(
                div()
                    .px_2()
                    .h(px(22.0))
                    .text_color(color(theme.text_dim))
                    .child(title),
            )
            .children(rows)
            .into_any_element(),
    )
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
    let label: String = format!("{name}:{line}  {}", place.preview);
    div()
        .id(("place", index))
        .px_2()
        .h(px(20.0))
        .hover(|el: gpui::StyleRefinement| el.bg(color(theme.overlay)))
        .child(label)
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
    let problems: Vec<(&Path, &Diagnostic)> = view.intel.problems();
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
            div()
                .px_2()
                .h(px(20.0))
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
