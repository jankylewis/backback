use super::super::model::ProjectTests;
use super::button;
use super::detail::label;
use crate::{color, tooltip, ExplorerView};
use bb_testkit::{Fqn, Outcome, SpecResult};
use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, AnyElement, Context, Window};
use std::path::{Path, PathBuf};

pub(super) fn project_row(
    path: PathBuf,
    project: &ProjectTests,
    counts: (usize, usize, usize, usize),
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    let name: String = path
        .file_stem()
        .map_or_else(String::new, |n: &std::ffi::OsStr| {
            n.to_string_lossy().into_owned()
        });
    let marker: &str = if project.expanded { "▾" } else { "▸" };
    let run_path: PathBuf = path.clone();
    div()
        .id(format!("test-project-{}", path.display()))
        .debug_selector(|| String::from("test-project"))
        .min_h(px(26.0))
        .px_2()
        .flex()
        .gap_2()
        .child(format!(
            "{marker} {name} ({} / {} / {} / {})",
            counts.0, counts.1, counts.2, counts.3
        ))
        .child(button(
            "Run",
            theme,
            cx,
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                super::super::run_project(view, run_path.clone(), window, cx);
            },
        ))
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                super::super::actions::toggle_project(view, path.clone(), window, cx);
            },
        ))
        .into_any_element()
}

pub(super) fn spec_rows(
    view: &ExplorerView,
    path: &Path,
    project: &ProjectTests,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> Vec<AnyElement> {
    let mut rows: Vec<AnyElement> = Vec::new();
    let mut last_namespace: String = String::new();
    let mut last_class: String = String::new();
    let mut last_method: String = String::new();
    for (fqn, result) in &project.specs {
        let parts: bb_testkit::FqnParts<'_> = fqn.parts();
        if !view.tests.filter.is_empty()
            && !fqn
                .as_str()
                .to_ascii_lowercase()
                .contains(&view.tests.filter.to_ascii_lowercase())
        {
            continue;
        }
        if parts.namespace != last_namespace {
            parts.namespace.clone_into(&mut last_namespace);
            last_class.clear();
            last_method.clear();
            rows.push(label(format!("  ▾ {}", parts.namespace), theme));
        }
        if parts.class != last_class {
            parts.class.clone_into(&mut last_class);
            last_method.clear();
            rows.push(class_row(path, parts.namespace, parts.class, theme, cx));
        }
        if fqn.as_str().contains('(') && fqn.method() != last_method {
            fqn.method().clone_into(&mut last_method);
            rows.push(label(format!("      ▾ {}", parts.spec), theme));
        }
        rows.push(spec_row(
            path.to_path_buf(),
            fqn.clone(),
            (result.as_ref(), is_running(view, path, fqn)),
            theme,
            cx,
        ));
    }
    rows
}

fn is_running(view: &ExplorerView, path: &Path, fqn: &Fqn) -> bool {
    view.tests
        .running_specs
        .get(path)
        .is_some_and(|names: &std::collections::BTreeSet<Fqn>| names.contains(fqn))
}

fn class_row(
    path: &Path,
    namespace: &str,
    class: &str,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    let class_path: PathBuf = path.to_path_buf();
    let owner: (String, String) = (namespace.to_owned(), class.to_owned());
    div()
        .id(format!("test-class-{}-{class}", path.display()))
        .debug_selector(|| String::from("test-class"))
        .min_h(px(24.0))
        .px_2()
        .flex()
        .justify_between()
        .child(format!("    ▾ {class}"))
        .child(
            div()
                .id("test-class-run")
                .debug_selector(|| String::from("test-class-run"))
                .child(button(
                    "Run",
                    theme,
                    cx,
                    move |view: &mut ExplorerView,
                          _: &gpui::ClickEvent,
                          window: &mut Window,
                          cx: &mut Context<ExplorerView>| {
                        super::super::run_class(view, class_path.clone(), &owner, window, cx);
                    },
                )),
        )
        .into_any_element()
}

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

fn spec_row(
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
        .child(div().text_color(color(ink)).child(icon))
        .child(label)
        .child(button(
            "▶",
            theme,
            cx,
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                super::super::run_spec(view, path.clone(), fqn.clone(), window, cx);
            },
        ))
        .child(debug_disabled(debug_id, theme))
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  _: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                select_spec(view, &select_path, &select_fqn, cx);
            },
        ))
        .into_any_element()
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
