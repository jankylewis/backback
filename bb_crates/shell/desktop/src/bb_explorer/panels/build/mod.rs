//! Build (Cmd+Shift+B): `dotnet build` on the solution in the background;
//! its errors and warnings list in the panel under the editor, and a click
//! goes to the place. Never with `--no-write`: a build writes bin/ and obj/.

mod bb_parse;
mod run;

use std::path::{Path, PathBuf};

use bb_intel::{DocPath, DocUri, Location, Span};
use bb_text::LspPoint;
use gpui::{AsyncWindowContext, Context, WeakEntity, Window};

pub(crate) use bb_parse::{issues, Issue, Severity};

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
    if view.tabs.no_write() {
        view.report(&"Build refused: --no-write keeps the solution unchanged");
        cx.notify();
        return;
    }
    let root: PathBuf = view.root.clone();
    let target: PathBuf =
        crate::intel::bb_worker::find_solution(&root).unwrap_or_else(|| root.clone());
    let running: run::Running = match run::start(&target) {
        Ok(running) => running,
        Err(error) => {
            view.report(&format!("Build could not run: {error}"));
            cx.notify();
            return;
        }
    };
    view.status = Some(format!("Building {}...", short(&target, &root)));
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
                    finish(v, result, &root);
                    cx.notify();
                },
            );
        },
    )
    .detach();
}

fn finish(view: &mut ExplorerView, result: Result<String, String>, root: &Path) {
    let output: String = match result {
        Ok(output) => output,
        Err(error) => {
            view.report(&format!("Build could not run: {error}"));
            return;
        }
    };
    let found: Vec<Issue> = issues(&output, root);
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
