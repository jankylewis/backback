//! pyright discovery, the interpreter choice and the Build menu's check on
//! the fixture, against the real tools of this machine where they exist.
#![cfg(test)]

use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use bb_python::fixtures::{self, Shape};
use bb_python::pyright::{self, Found, Inputs, Source};
use bb_python::run::check::{self, Issue, Level};
use bb_python::{interpreter_for, Interpreter, InterpreterSource, PythonError};

fn fixture(shape: Shape) -> tempfile::TempDir {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    fixtures::write(dir.path(), shape).unwrap();
    dir
}

fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path: PathBuf = dir.join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

fn files_under(root: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut todo: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = todo.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path: PathBuf = entry.unwrap().path();
            if path.is_dir() {
                todo.push(path.clone());
            }
            found.push(path.strip_prefix(root).unwrap().to_path_buf());
        }
    }
    found.sort();
    found
}

#[test]
fn the_fake_venv_folder_uses_its_venv_and_the_plain_folder_uses_python3_on_path() {
    let root: tempfile::TempDir = fixture(Shape::Clean);
    let path: Option<OsString> = std::env::var_os("PATH");
    let venv_file: PathBuf = root.path().join(fixtures::VENV_FOLDER).join("app.py");
    let plain_file: PathBuf = root.path().join(fixtures::PLAIN_FOLDER).join("tool.py");
    let in_venv: Interpreter = interpreter_for(&venv_file, root.path(), path.as_ref());
    let in_plain: Interpreter = interpreter_for(&plain_file, root.path(), path.as_ref());
    let venv: PathBuf = root.path().join(fixtures::VENV_FOLDER).join(".venv");
    assert_eq!(in_venv.source, InterpreterSource::Venv(venv.clone()));
    assert_eq!(in_venv.program, venv.join("bin/python"));
    assert_eq!(in_plain.source, InterpreterSource::Path);
    assert!(
        in_plain.program.ends_with("python3"),
        "{}",
        in_plain.program.display()
    );
}

#[test]
fn a_server_whose_command_line_answers_is_found_and_one_without_is_not() {
    let bin: tempfile::TempDir = tempfile::tempdir().unwrap();
    script(bin.path(), "pyright-langserver", "exit 0");
    let lone: Inputs = Inputs {
        env: None,
        path: Some(bin.path().as_os_str().to_owned()),
    };
    assert!(matches!(
        pyright::locate(&lone),
        Err(PythonError::PyrightMissing)
    ));
    script(bin.path(), "pyright", "echo 'pyright 1.1.400'");
    let found: Found = pyright::locate(&lone).unwrap();
    assert_eq!(
        (
            found.server,
            found.cli,
            found.source,
            found.version.as_str()
        ),
        (
            bin.path().join("pyright-langserver"),
            bin.path().join("pyright"),
            Source::Path,
            "pyright 1.1.400"
        )
    );
}

#[test]
fn an_override_that_does_not_run_is_reported_never_skipped() {
    let inputs: Inputs = Inputs {
        env: Some(OsString::from("/nonexistent/pyright-langserver")),
        path: std::env::var_os("PATH"),
    };
    let error: PythonError = pyright::locate(&inputs).unwrap_err();
    assert!(
        matches!(error, PythonError::PyrightBroken { .. }),
        "{error}"
    );
}

#[test]
fn the_syntax_check_names_the_broken_line_and_writes_nothing_into_the_project() {
    let root: tempfile::TempDir = fixture(Shape::Broken);
    let before: Vec<PathBuf> = files_under(root.path());
    let python: Interpreter =
        interpreter_for(root.path(), root.path(), std::env::var_os("PATH").as_ref());
    let found: Vec<Issue> = check::syntax(&python.program, root.path()).unwrap();
    let shown: Vec<(PathBuf, u32, Level)> = found
        .into_iter()
        .map(|i: Issue| (i.path, i.line, i.level))
        .collect();
    assert_eq!(
        shown,
        vec![(
            root.path().join(fixtures::BROKEN_FILE),
            fixtures::BROKEN_LINE,
            Level::Error
        )]
    );
    assert_eq!(files_under(root.path()), before);
}

#[test]
fn pyright_checks_the_fixture_or_discovery_names_the_install_hint() {
    let base: tempfile::TempDir = fixture(Shape::Broken);
    let root: PathBuf = base.path().canonicalize().unwrap();
    let before: Vec<PathBuf> = files_under(&root);
    let python: Interpreter = interpreter_for(&root, &root, std::env::var_os("PATH").as_ref());
    let outcome: Result<Vec<Issue>, String> = match pyright::locate(&Inputs::from_env()) {
        Ok(found) => check::pyright(&found.cli, &python.program, &root)
            .map_err(|e: PythonError| e.to_string()),
        Err(missing) => Err(missing.to_string()),
    };
    match outcome {
        Ok(found) => assert!(
            found
                .iter()
                .any(|i: &Issue| i.path == root.join(fixtures::BROKEN_FILE)
                    && i.line == fixtures::BROKEN_LINE
                    && i.level == Level::Error),
            "{found:?}"
        ),
        Err(hint) => {
            assert!(
                std::env::var_os("BACKBACK_REQUIRE_TOOLS").is_none(),
                "BACKBACK_REQUIRE_TOOLS is set: {hint}"
            );
            assert_eq!(hint, pyright::MISSING_HINT);
        }
    }
    assert_eq!(files_under(&root), before);
}

#[test]
fn pyright_output_lists_errors_and_warnings_one_based_and_drops_information() {
    let json: &str = r#"{"generalDiagnostics":[
 {"file":"/w/a.py","severity":"error","message":"bad","range":{"start":{"line":3,"character":4},"end":{"line":3,"character":5}}},
 {"file":"/w/b.py","severity":"warning","message":"meh","range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}}},
 {"file":"/w/c.py","severity":"information","message":"fyi","range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}}}]}"#;
    let shown: Vec<(PathBuf, u32, u32, Level)> = check::pyright_issues(json)
        .unwrap()
        .into_iter()
        .map(|i: Issue| (i.path, i.line, i.column, i.level))
        .collect();
    assert_eq!(
        shown,
        vec![
            (PathBuf::from("/w/a.py"), 4, 5, Level::Error),
            (PathBuf::from("/w/b.py"), 1, 1, Level::Warning),
        ]
    );
}
