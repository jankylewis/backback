//! Build > Python Check (BBEOD28): pyright's diagnostics on the window's
//! folder with its interpreter; when pyright is missing, the interpreter's
//! own syntax check (and the install hint in the title). Both list
//! `file:line` like a .NET build, a click away. Nothing is written into the
//! project: no bytecode, no cache.

use std::path::{Path, PathBuf};
use std::process::Command;

use bb_python::pyright::{self, Inputs};
use bb_python::run::check::{self, Level};
use gpui::{Context, Window};

use super::{launch, run, Issue, Reader, Severity};
use crate::ExplorerView;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct PythonCheck;

pub(crate) fn on_python_check(
    view: &mut ExplorerView,
    _: &PythonCheck,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    python_check(view, window, cx);
}

/// What a check runs: pyright when it runs here, else the syntax check.
pub(crate) fn plan(root: &Path, inputs: &Inputs) -> (Command, String, Reader) {
    let python: PathBuf = crate::intel::bb_worker::interpreter(root);
    match pyright::locate(inputs) {
        Ok(found) => {
            let cli: PathBuf = found.cli;
            (
                check::pyright_command(&cli, &python, root),
                String::from("pyright..."),
                pyright_reader,
            )
        }
        Err(missing) => (
            check::syntax_command(&python, root),
            format!("Python syntax check ({missing})..."),
            syntax_reader,
        ),
    }
}

/// Check the folder while Python is active; else say why.
pub(crate) fn python_check(
    view: &mut ExplorerView,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    if !view.langs.langs.allows(bb_lang::Tool::Pyright) {
        view.report(&"Build: Python is not active here");
        cx.notify();
        return;
    }
    let (command, title, read): (Command, String, Reader) = plan(&view.root, &Inputs::from_env());
    let started: Result<run::Running, String> = run::spawn(command, "python check");
    launch(view, (started, title, read), window, cx);
}

fn pyright_reader(output: &str, _root: &Path) -> Vec<Issue> {
    match check::pyright_issues(output) {
        Ok(found) => found.into_iter().map(issue).collect(),
        Err(error) => vec![Issue {
            path: PathBuf::new(),
            line: 1,
            column: 1,
            severity: Severity::Error,
            text: error.to_string(),
        }],
    }
}

fn syntax_reader(output: &str, _root: &Path) -> Vec<Issue> {
    check::syntax_issues(output)
        .into_iter()
        .map(issue)
        .collect()
}

fn issue(found: check::Issue) -> Issue {
    Issue {
        path: found.path,
        line: found.line,
        column: found.column,
        severity: match found.level {
            Level::Error => Severity::Error,
            Level::Warning => Severity::Warning,
        },
        text: found.text,
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{pyright_reader, syntax_reader, Issue, Severity};

    #[test]
    fn a_syntax_error_lists_as_an_error_at_its_line() {
        let line: &str = r#"{"file": "/w/shop/broken.py", "line": 2, "column": 12, "message": "invalid syntax"}"#;
        let found: Vec<(PathBuf, u32, Severity)> = syntax_reader(line, Path::new("/w"))
            .into_iter()
            .map(|i: Issue| (i.path, i.line, i.severity))
            .collect();
        assert_eq!(
            found,
            vec![(PathBuf::from("/w/shop/broken.py"), 2, Severity::Error)]
        );
    }

    #[test]
    fn check_runs_pyright_when_it_runs_here_else_the_syntax_check_with_the_hint() {
        use std::os::unix::fs::PermissionsExt;
        let bin: tempfile::TempDir = tempfile::tempdir().unwrap();
        let root: tempfile::TempDir = tempfile::tempdir().unwrap();
        let empty: bb_python::pyright::Inputs = bb_python::pyright::Inputs {
            env: None,
            path: Some(bin.path().as_os_str().to_owned()),
        };
        let (_, missing, _): (std::process::Command, String, super::Reader) =
            super::plan(root.path(), &empty);
        assert_eq!(
            missing,
            format!(
                "Python syntax check ({})...",
                bb_python::pyright::MISSING_HINT
            )
        );
        for (name, body) in [
            ("pyright-langserver", "exit 0"),
            ("pyright", "echo pyright 1.1.400"),
        ] {
            let file: PathBuf = bin.path().join(name);
            std::fs::write(&file, format!("#!/bin/sh\n{body}\n")).unwrap();
            std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let (command, title, _): (std::process::Command, String, super::Reader) =
            super::plan(root.path(), &empty);
        assert_eq!(
            (title.as_str(), command.get_program()),
            ("pyright...", bin.path().join("pyright").as_os_str())
        );
    }

    #[test]
    fn unreadable_pyright_output_is_shown_never_dropped() {
        let found: Vec<Issue> = pyright_reader("not json", Path::new("/w"));
        assert_eq!(found.len(), 1);
        assert!(found.iter().all(|i: &Issue| i.text.contains("pyright")));
    }
}
