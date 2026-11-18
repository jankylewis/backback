//! Build (Cmd+Shift+B): `dotnet build` on the solution in the background;
//! its errors and warnings list in the panel under the editor, and a click
//! goes to the place. Never with `--no-write`: a build writes bin/ and obj/.

mod bb_parse;
mod cargo;
mod python;
mod run;

use std::path::{Path, PathBuf};

use bb_intel::{DocPath, DocUri, Location, Span};
use bb_text::LspPoint;
use gpui::{AsyncWindowContext, Context, WeakEntity, Window};

pub(crate) use bb_parse::{issues, Issue, Severity};
pub(crate) use cargo::{on_cargo_build, on_cargo_check, CargoBuild, CargoCheck};
pub(crate) use python::{on_python_check, PythonCheck};

use crate::intel::bb_worker::protocol::Place;
use crate::intel::model::popups::{Pick, PlacesPanel};
use crate::ExplorerView;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct BuildSolution;

pub(crate) fn on_build(
    view: &mut ExplorerView,
    _: &BuildSolution,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    if !view.langs.langs.allows(bb_lang::Tool::Roslyn) {
        let rust: bool = view.langs.langs.allows(bb_lang::Tool::Cargo)
            && crate::intel::bb_worker::find_cargo(&view.root).is_some();
        if !rust && view.langs.langs.allows(bb_lang::Tool::Pyright) {
            python::python_check(view, window, cx);
        } else {
            cargo::cargo_build(view, bb_cargo::BuildKind::Build, window, cx);
        }
        return;
    }
    if refused(view, cx) {
        return;
    }
    let root: PathBuf = view.root.clone();
    let target: PathBuf =
        crate::intel::bb_worker::find_solution(&root).unwrap_or_else(|| root.clone());
    let started: Result<run::Running, String> = run::start(&target);
    let title: String = format!("Building {}...", short(&target, &root));
    launch(view, (started, title, issues), window, cx);
}

/// `--no-write` keeps the workspace unchanged, and a build writes into it.
fn refused(view: &mut ExplorerView, cx: &mut Context<ExplorerView>) -> bool {
    if view.tabs.no_write() {
        view.report(&"Build refused: --no-write keeps the solution unchanged");
        cx.notify();
    }
    view.tabs.no_write()
}

/// How a build's output becomes issues.
type Reader = fn(&str, &Path) -> Vec<Issue>;

/// Wait for a started build in the background, then list its issues.
fn launch(
    view: &mut ExplorerView,
    (started, title, read): (Result<run::Running, String>, String, Reader),
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let root: PathBuf = view.root.clone();
    let running: run::Running = match started {
        Ok(running) => running,
        Err(error) => {
            view.report(&format!("Build could not run: {error}"));
            cx.notify();
            return;
        }
    };
    view.status = Some(title);
    cx.notify();
    let build: gpui::Task<Result<String, String>> = cx
        .background_executor()
        .spawn(async move { run::wait(running) });
    cx.spawn_in(
        window,
        async move |view: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext| {
            let result: Result<String, String> = build.await;
            let _closed: gpui::Result<()> = view.update(
                cx,
                |v: &mut ExplorerView, cx: &mut Context<ExplorerView>| {
                    finish(v, result.map(|out: String| read(&out, &root)));
                    cx.notify();
                },
            );
        },
    )
    .detach();
}

pub(super) fn finish(view: &mut ExplorerView, result: Result<Vec<Issue>, String>) {
    let found: Vec<Issue> = match result {
        Ok(found) => found,
        Err(error) => {
            view.report(&format!("Build could not run: {error}"));
            return;
        }
    };
    let errors: usize = found
        .iter()
        .filter(|i: &&Issue| i.severity == Severity::Error)
        .count();
    let warnings: usize = found.len() - errors;
    let title: String = if errors == 0 {
        format!("Build succeeded: {}", count(warnings, "warning"))
    } else {
        format!(
            "Build failed: {}, {}",
            count(errors, "error"),
            count(warnings, "warning")
        )
    };
    view.status = Some(title.clone());
    view.intel.references = Some(PlacesPanel {
        title,
        places: found.iter().filter_map(place).collect(),
        pick: Pick::default(),
    });
}

fn place(issue: &Issue) -> Option<Place> {
    let at: LspPoint = LspPoint {
        line: issue.line.saturating_sub(1),
        character: issue.column.saturating_sub(1),
    };
    let kind: &str = match issue.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    };
    Some(Place {
        location: Location {
            uri: DocUri::File(DocPath::new(issue.path.clone()).ok()?),
            span: Span { start: at, end: at },
        },
        preview: format!("{kind} {}", issue.text),
    })
}

/// `1 error`, `2 errors`.
fn count(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}

fn short(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}
