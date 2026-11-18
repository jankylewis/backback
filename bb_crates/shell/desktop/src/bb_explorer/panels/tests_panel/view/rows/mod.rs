use crate::tests_panel::model::ProjectTests;
use crate::tests_panel::Group;
use crate::ExplorerView;
use bb_testkit::Fqn;
use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, AnyElement, Context, Window};
use std::path::{Path, PathBuf};

#[path = "bb_groups.rs"]
mod groups;
mod menu;
mod spec;
pub(crate) use menu::{menu as test_menu, TestMenu};
pub(crate) use spec::open_spec;
#[cfg(test)]
mod window_tests;

pub(super) fn project_row(
    path: PathBuf,
    project: &ProjectTests,
    counts: (usize, usize, usize, usize),
    (theme, running): (Theme, bool),
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    let name: String = project_name(&path);
    let marker: &str = if project.expanded { "▾" } else { "▸" };
    let run_path: PathBuf = path.clone();
    div()
        .id(format!("test-project-{}", path.display()))
        .debug_selector(|| String::from("test-project"))
        .min_h(px(26.0))
        .px_2()
        .flex()
        .gap_2()
        .cursor_pointer()
        .child({
            // One selector per row: every Cargo member's file is
            // `Cargo.toml`, and a shared one hits whichever row drew last.
            let selector: String = format!("test-project-{name}");
            div().debug_selector(move || selector).child(format!(
                "{marker} {name} ({} / {} / {} / {})",
                counts.0, counts.1, counts.2, counts.3
            ))
        })
        .child(groups::run_or_stop(
            running,
            theme,
            cx,
            move |view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>| {
                crate::tests_panel::run_project(view, run_path.clone(), window, cx);
            },
        ))
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                crate::tests_panel::toggle_project(view, path.clone(), window, cx);
            },
        ))
        .into_any_element()
}

/// The last group heads written, so each head is written once.
#[derive(Default)]
struct Last {
    namespace: String,
    class: String,
    method: String,
}

/// The tree under an open project: namespace, class and parameterised
/// method heads, each foldable (BB-BUG-024), then the specs left open.
pub(super) fn spec_rows(
    view: &ExplorerView,
    path: &Path,
    project: &ProjectTests,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> Vec<AnyElement> {
    let mut rows: Vec<AnyElement> = Vec::new();
    let mut last: Last = Last::default();
    let filter: String = crate::tests_panel::find::lowered(&view.tests.filter);
    for (fqn, result) in &project.specs {
        if !crate::tests_panel::find::holds(fqn, &filter) {
            continue;
        }
        if !heads(view, (path, fqn), &mut last, (theme, cx), &mut rows) {
            continue;
        }
        let run: menu::Run = menu::Run::of(view, is_running(view, path, fqn), fqn);
        rows.push(spec::spec_row(
            path.to_path_buf(),
            fqn.clone(),
            (result.as_ref(), run),
            theme,
            cx,
        ));
    }
    rows
}

/// Push the heads `fqn` opens; whether its own row shows (no folded head).
fn heads(
    view: &ExplorerView,
    (path, fqn): (&Path, &Fqn),
    last: &mut Last,
    (theme, cx): (Theme, &mut Context<ExplorerView>),
    rows: &mut Vec<AnyElement>,
) -> bool {
    let parts: bb_testkit::FqnParts<'_> = fqn.parts();
    let namespace: Group = Group::Namespace(parts.namespace.to_owned());
    if parts.namespace != last.namespace {
        parts.namespace.clone_into(&mut last.namespace);
        last.class.clear();
        last.method.clear();
        rows.push(groups::group_row(view, path, namespace.clone(), theme, cx));
    }
    let owner: String = if parts.namespace.is_empty() {
        parts.class.to_owned()
    } else {
        format!("{}.{}", parts.namespace, parts.class)
    };
    if view.tests.is_collapsed(path, &namespace) {
        return false;
    }
    let class: Group = Group::Class(owner);
    if parts.class != last.class {
        parts.class.clone_into(&mut last.class);
        last.method.clear();
        rows.push(groups::group_row(view, path, class.clone(), theme, cx));
    }
    if view.tests.is_collapsed(path, &class) {
        return false;
    }
    if !fqn.as_str().contains('(') {
        return true;
    }
    let method: Group = Group::Method(fqn.method().to_owned());
    if fqn.method() != last.method {
        fqn.method().clone_into(&mut last.method);
        rows.push(groups::group_row(view, path, method.clone(), theme, cx));
    }
    !view.tests.is_collapsed(path, &method)
}

pub(super) fn is_running(view: &ExplorerView, path: &Path, fqn: &Fqn) -> bool {
    view.tests
        .running_specs
        .get(path)
        .is_some_and(|names: &std::collections::BTreeSet<Fqn>| names.contains(fqn))
}

/// Whether any spec of `path` whose name starts with `prefix` is running.
pub(super) fn any_running(view: &ExplorerView, path: &Path, prefix: &str) -> bool {
    view.tests
        .running_specs
        .get(path)
        .is_some_and(|names: &std::collections::BTreeSet<Fqn>| {
            names
                .iter()
                .any(|name: &Fqn| name.as_str().starts_with(prefix))
        })
}

/// The rows at their full width, scrolled both ways, with a bar along the
/// bottom while one is wider than the panel (BB-FEAT-085).
pub(super) fn rows_box(
    view: &ExplorerView,
    rows: Vec<AnyElement>,
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    let wide: Vec<gpui::Div> = rows
        .into_iter()
        .map(|row: AnyElement| {
            div()
                .min_w_full()
                .flex_none()
                .whitespace_nowrap()
                .child(row)
        })
        .collect();
    div()
        .relative()
        .flex_1()
        .min_h(px(0.0))
        .child(
            div()
                .id("test-rows")
                .debug_selector(|| "test-rows".to_owned())
                .size_full()
                .overflow_scroll()
                .track_scroll(&view.tests.rows_scroll)
                .flex()
                .flex_col()
                .items_start()
                .children(wide),
        )
        .children(crate::editor::panel_bar(
            view,
            crate::editor::PanelBar::Tests,
            cx,
        ))
        .into_any_element()
}

/// The row's name: a .NET project's file stem; a Cargo member's or pytest
/// root's folder, since their files are `Cargo.toml` or `pyproject.toml`.
pub(super) fn project_name(path: &Path) -> String {
    let named: Option<&std::ffi::OsStr> = match crate::tests_panel::model::ProjectKind::of(path) {
        crate::tests_panel::model::ProjectKind::Dotnet => path.file_stem(),
        crate::tests_panel::model::ProjectKind::Cargo
        | crate::tests_panel::model::ProjectKind::Pytest => path.parent().and_then(Path::file_name),
    };
    named.map_or_else(String::new, |n: &std::ffi::OsStr| {
        n.to_string_lossy().into_owned()
    })
}
