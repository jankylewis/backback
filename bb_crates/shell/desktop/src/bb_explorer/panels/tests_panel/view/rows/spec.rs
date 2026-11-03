//! One spec's row: its verdict, name and time; a click shows its result
//! and opens its source (BB-BUG-025); ▶ runs it, ■ stops it (BB-BUG-026).

use super::super::button;
use crate::tests_panel::model::ProjectTests;
use crate::{color, tooltip, ExplorerView};
use bb_testkit::{Fqn, Outcome, SpecResult};
use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, AnyElement, AsyncWindowContext, Context, WeakEntity, Window};
use std::path::{Path, PathBuf};

fn spec_display(
    fqn: &Fqn,
    result: Option<&SpecResult>,
    running: bool,
    theme: Theme,
) -> (&'static str, String, bb_ui::Rgb) {
    let icon: &str = if running {
        "◌"
    } else {
        match result.map(|r: &SpecResult| r.outcome) {
            Some(Outcome::Passed) => "✓",
            Some(Outcome::Failed) => "✕",
            Some(Outcome::NotExecuted | Outcome::Inconclusive) => "○",
            None => "·",
        }
    };
    let name: &str = if fqn.as_str().contains('(') {
        fqn.as_str().get(fqn.method().len()..).unwrap_or_default()
    } else {
        fqn.parts().spec
    };
    let duration: String = result.map_or_else(String::new, |r: &SpecResult| {
        format!("  {} ms", r.duration.as_millis())
    });
    let ink: bb_ui::Rgb = if running {
        theme.accent
    } else {
        match result.map(|r: &SpecResult| r.outcome) {
            Some(Outcome::Passed) => theme.ok,
            Some(Outcome::Failed) => theme.error,
            Some(Outcome::NotExecuted | Outcome::Inconclusive) => theme.warn,
            None => theme.text_dim,
        }
    };
    (icon, format!("      {name}{duration}"), ink)
}

pub(super) fn spec_row(
    path: PathBuf,
    fqn: Fqn,
    (result, running): (Option<&SpecResult>, bool),
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    let (icon, label, ink): (&str, String, bb_ui::Rgb) = spec_display(&fqn, result, running, theme);
    let select_path: PathBuf = path.clone();
    let select_fqn: Fqn = fqn.clone();
    let debug_name: String = format!("test-spec-{}", fqn.as_str());
    let debug_id: String = format!("test-debug-{}", fqn.as_str());
    div()
        .id(format!("test-spec-{}", fqn.as_str()))
        .debug_selector(move || debug_name)
        .min_h(px(24.0))
        .px_2()
        .flex()
        .justify_between()
        .cursor_pointer()
        .child(div().text_color(color(ink)).child(icon))
        .child(label)
        .child(play_or_stop(path, fqn, running, theme, cx))
        .child(debug_disabled(debug_id, theme))
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                open_spec(view, &select_path, &select_fqn, window, cx);
            },
        ))
        .into_any_element()
}

fn play_or_stop(
    path: PathBuf,
    fqn: Fqn,
    running: bool,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    let selector: String = format!("test-stop-{}", fqn.as_str());
    if running {
        return div()
            .id("test-spec-stop")
            .debug_selector(move || selector)
            .child(button(
                "■",
                theme,
                cx,
                |view: &mut ExplorerView,
                 _: &gpui::ClickEvent,
                 _: &mut Window,
                 cx: &mut Context<ExplorerView>| {
                    crate::tests_panel::stop(view, cx);
                },
            ));
    }
    div().id("test-spec-play").child(button(
        "▶",
        theme,
        cx,
        move |view: &mut ExplorerView,
              _: &gpui::ClickEvent,
              window: &mut Window,
              cx: &mut Context<ExplorerView>| {
            crate::tests_panel::run_spec(view, path.clone(), fqn.clone(), window, cx);
        },
    ))
}

fn debug_disabled(id: String, theme: Theme) -> impl IntoElement {
    div()
        .id(id)
        .text_color(color(theme.text_dim))
        .child("Debug")
        .tooltip(tooltip::path(
            String::from("Available once the debugger lands"),
            theme,
        ))
}

/// What a click on a spec's row does: show its result, open its source.
pub(crate) fn open_spec(
    view: &mut ExplorerView,
    path: &Path,
    fqn: &Fqn,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    select_spec(view, path, fqn, cx);
    open_source(path, fqn, window, cx);
}

fn select_spec(view: &mut ExplorerView, path: &Path, fqn: &Fqn, cx: &mut Context<ExplorerView>) {
    view.tests.selected = Some((path.to_path_buf(), fqn.clone()));
    view.tests.detail = view
        .tests
        .model
        .projects()
        .get(path)
        .and_then(|project: &ProjectTests| project.specs.get(fqn))
        .and_then(Option::clone);
    cx.notify();
}

/// Find the spec's method in its project's `.cs` files off the UI thread,
/// then open it there; no declaring file is said in the status bar.
fn open_source(project: &Path, fqn: &Fqn, window: &mut Window, cx: &mut Context<ExplorerView>) {
    let dir: PathBuf = project.parent().map(Path::to_path_buf).unwrap_or_default();
    let name: Fqn = fqn.clone();
    let job: gpui::Task<Option<(PathBuf, usize)>> = cx
        .background_executor()
        .spawn(async move { bb_testkit::find_source(&dir, &name) });
    let shown: String = fqn.as_str().to_owned();
    cx.spawn_in(
        window,
        async move |view: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext| {
            let found: Option<(PathBuf, usize)> = job.await;
            view.update_in(
                cx,
                |view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>| {
                    match found {
                        Some((file, line)) => {
                            crate::tests_panel::navigation::jump_to(
                                view,
                                (&file, line),
                                window,
                                cx,
                            );
                        }
                        None => view.report(&format!("No source found for {shown}")),
                    }
                },
            )
            .ok();
        },
    )
    .detach();
}
