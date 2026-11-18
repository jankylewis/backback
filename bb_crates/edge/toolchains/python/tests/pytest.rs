//! pytest on the fixture with this machine's `python3`: the full list and
//! run where pytest is installed, else the visible install hint. Either
//! way nothing is written into the project.
#![cfg(test)]

use std::path::{Path, PathBuf};

use bb_python::fixtures::{self, Shape};
use bb_python::run::bb_pytest::{self, missing_hint};
use bb_python::{interpreter_for, Interpreter, PythonError, TestRoot};
use bb_testkit::{Fqn, Leaf, Outcome, RunReport, Selection, SpecList, StopFlag};

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

fn outcomes(report: &RunReport) -> Vec<(String, Outcome)> {
    report
        .leaves
        .iter()
        .filter_map(|leaf: &Leaf| match leaf {
            Leaf::Ran(result) => Some((result.fqn.as_str().to_owned(), result.outcome)),
            Leaf::Missing(_) => None,
        })
        .collect()
}

fn listed_then_run(root: &TestRoot, file: &Path) -> Result<(SpecList, RunReport), PythonError> {
    let list: SpecList = bb_pytest::list(root, file)?;
    let all: RunReport = bb_pytest::run(root, &Selection::default(), &StopFlag::new())?;
    Ok((list, all))
}

fn assert_full_run(root: &TestRoot, list: &SpecList, all: &RunReport) {
    let names: Vec<&str> = list.specs.iter().map(Fqn::as_str).collect();
    assert_eq!(names, vec![PASSES, FAILS, SKIPPED]);
    assert_eq!(
        outcomes(all),
        vec![
            (String::from(PASSES), Outcome::Passed),
            (String::from(FAILS), Outcome::Failed),
            (String::from(SKIPPED), Outcome::NotExecuted),
        ]
    );
    let one: Selection = Selection {
        specs: vec![Fqn::new(FAILS).unwrap()],
        category: None,
    };
    let single: RunReport = bb_pytest::run(root, &one, &StopFlag::new()).unwrap();
    assert_eq!(
        outcomes(&single),
        vec![(String::from(FAILS), Outcome::Failed)]
    );
}

const PASSES: &str = "tests.test_cart.test_passes";
const FAILS: &str = "tests.test_cart.test_fails";
const SKIPPED: &str = "tests.test_cart.test_skipped";

#[test]
fn pytest_lists_and_runs_pass_fail_skip_or_names_the_install_command() {
    let project: tempfile::TempDir = tempfile::tempdir().unwrap();
    let scratch: tempfile::TempDir = tempfile::tempdir().unwrap();
    fixtures::write(project.path(), Shape::Clean).unwrap();
    let before: Vec<PathBuf> = files_under(project.path());
    let python: Interpreter = interpreter_for(
        project.path(),
        project.path(),
        std::env::var_os("PATH").as_ref(),
    );
    let root: TestRoot = TestRoot {
        dir: project.path().to_path_buf(),
        interpreter: python.program.clone(),
        scratch: scratch.path().to_path_buf(),
    };
    match listed_then_run(&root, &project.path().join("pyproject.toml")) {
        Ok((list, all)) => assert_full_run(&root, &list, &all),
        Err(PythonError::PytestMissing(hint)) => assert_eq!(hint, missing_hint(&python.program)),
        Err(other) => panic!("neither a run nor the missing hint: {other}"),
    }
    assert_eq!(files_under(project.path()), before);
    assert_eq!(files_under(scratch.path()), Vec::<PathBuf>::new());
}

#[test]
fn a_folder_is_a_pytest_root_by_its_root_file() {
    let project: tempfile::TempDir = tempfile::tempdir().unwrap();
    fixtures::write(project.path(), Shape::Clean).unwrap();
    assert_eq!(
        bb_pytest::root_file(project.path()),
        Some(project.path().join("pyproject.toml"))
    );
    assert_eq!(
        bb_pytest::root_file(&project.path().join(fixtures::PLAIN_FOLDER)),
        None
    );
}

#[test]
fn collect_only_output_becomes_the_panel_names_in_order() {
    let stdout: &str = "tests/test_cart.py::test_passes\ntests/test_cart.py::K::test_p[1.5]\n\n2 tests collected in 0.01s\n";
    let names: Vec<String> = bb_pytest::collected(stdout)
        .into_iter()
        .map(|f: Fqn| f.as_str().to_owned())
        .collect();
    assert_eq!(
        names,
        vec![
            "tests.test_cart.test_passes",
            "tests.test_cart.K.test_p(1.5)"
        ]
    );
}
