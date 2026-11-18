//! Build > Cargo Build / Cargo Check (BBEOD27): `cargo build` or `cargo
//! check` on the window's Cargo workspace, its JSON messages listed like a
//! .NET build's, each a click away from its file and line.

use std::path::{Path, PathBuf};

use bb_cargo::project::messages::{self, Level, Message};
use bb_cargo::BuildKind;
use gpui::{Context, Window};

use super::{launch, refused, run, Issue, Severity};
use crate::ExplorerView;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct CargoBuild;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct CargoCheck;

pub(crate) fn on_cargo_build(
    view: &mut ExplorerView,
    _: &CargoBuild,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    cargo_build(view, BuildKind::Build, window, cx);
}

pub(crate) fn on_cargo_check(
    view: &mut ExplorerView,
    _: &CargoCheck,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    cargo_build(view, BuildKind::Check, window, cx);
}

/// Run `kind` on the workspace when Rust is active and it holds a
/// `Cargo.toml`; else say why in the status bar.
pub(crate) fn cargo_build(
    view: &mut ExplorerView,
    kind: BuildKind,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let root: Option<PathBuf> = crate::intel::bb_worker::find_cargo(&view.root);
    let Some(root): Option<PathBuf> =
        root.filter(|_: &PathBuf| view.langs.start(bb_lang::Tool::Cargo))
    else {
        view.report(&"Build: no active Rust workspace (Cargo.toml) here");
        cx.notify();
        return;
    };
    if refused(view, cx) {
        return;
    }
    let title: String = format!("cargo {}...", kind.verb());
    let started: Result<run::Running, String> = run::spawn(messages::command(&root, kind), "cargo");
    launch(view, (started, title, issues), window, cx);
}

/// The issues in `cargo --message-format=json` output run in `root`.
pub(crate) fn issues(output: &str, root: &Path) -> Vec<Issue> {
    messages::messages(output, root)
        .into_iter()
        .map(|message: Message| Issue {
            path: message.path,
            line: message.line,
            column: message.column,
            severity: match message.level {
                Level::Error => Severity::Error,
                Level::Warning => Severity::Warning,
            },
            text: message.text,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{issues, Issue, Severity};

    const WARNING: &str = r#"{"reason":"compiler-message","message":{"level":"warning","message":"unused variable: `x`","spans":[{"file_name":"alpha/src/math.rs","line_start":7,"column_start":9,"is_primary":true}]}}"#;
    const ERROR: &str = r#"{"reason":"compiler-message","message":{"level":"error","message":"mismatched types","spans":[{"file_name":"beta/src/lib.rs","line_start":4,"column_start":5,"is_primary":true}]}}"#;

    #[test]
    fn a_cargo_warning_lists_as_a_warning_and_an_error_as_an_error() {
        let found: Vec<Issue> = issues(&format!("{WARNING}\n{ERROR}"), Path::new("/w"));
        let shown: Vec<(PathBuf, u32, Severity)> = found
            .into_iter()
            .map(|i: Issue| (i.path, i.line, i.severity))
            .collect();
        assert_eq!(
            shown,
            vec![
                (PathBuf::from("/w/alpha/src/math.rs"), 7, Severity::Warning),
                (PathBuf::from("/w/beta/src/lib.rs"), 4, Severity::Error),
            ]
        );
    }
}
