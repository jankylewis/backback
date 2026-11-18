//! The fixture workspace through real cargo: members, Build messages and
//! tests (list, run one, run all), all built in a scratch target folder.
#![cfg(test)]

use std::path::{Path, PathBuf};

use bb_cargo::fixtures::{self, Shape, BROKEN_LINE, TESTS};
use bb_cargo::project::messages::{self, BuildKind, Level, Message};
use bb_cargo::run::bb_proc::{self, Output};
use bb_cargo::run::cargo_test::{self, TestTarget};
use bb_cargo::{CargoWorkspace, Member, METADATA_DEADLINE, TEST_DEADLINE};
use bb_testkit::{Fqn, Leaf, Outcome, RunReport, Selection, SpecList, StopFlag};

fn workspace(shape: Shape) -> (tempfile::TempDir, PathBuf) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = base.path().join("ws");
    fixtures::write(&root, shape).unwrap();
    (base, root.canonicalize().unwrap())
}

fn target(base: &tempfile::TempDir, root: &Path) -> TestTarget {
    TestTarget {
        root: root.to_path_buf(),
        package: String::from("alpha"),
        target_dir: base.path().join("scratch-target"),
    }
}

fn outcome(report: &RunReport, name: &str) -> Option<Outcome> {
    report.leaves.iter().find_map(|leaf: &Leaf| match leaf {
        Leaf::Ran(result) if result.fqn.as_str() == name => Some(result.outcome),
        Leaf::Ran(_) | Leaf::Missing(_) => None,
    })
}

#[test]
fn metadata_lists_both_members_without_writing_the_project() {
    let (_base, root): (tempfile::TempDir, PathBuf) = workspace(Shape::Clean);
    let found: CargoWorkspace = bb_cargo::members(&root, METADATA_DEADLINE).unwrap();
    let names: Vec<(&str, PathBuf)> = found
        .members
        .iter()
        .map(|m: &Member| (m.name.as_str(), m.manifest.clone()))
        .collect();
    assert_eq!(
        names,
        vec![
            ("alpha", root.join("alpha/Cargo.toml")),
            ("beta", root.join("beta/Cargo.toml")),
        ]
    );
    assert!(!root.join("target").exists());
    assert!(!root.join("Cargo.lock").exists());
}

#[test]
fn metadata_outside_a_cargo_workspace_is_a_typed_failure() {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let result: Result<CargoWorkspace, bb_cargo::CargoError> =
        bb_cargo::members(base.path(), METADATA_DEADLINE);
    assert!(
        matches!(result, Err(bb_cargo::CargoError::Failed { .. })),
        "{result:?}"
    );
}

#[test]
fn check_of_the_broken_member_names_the_error_at_its_file_and_line() {
    let (base, root): (tempfile::TempDir, PathBuf) = workspace(Shape::Broken);
    let mut command: std::process::Command = messages::command(&root, BuildKind::Check);
    command.env("CARGO_TARGET_DIR", base.path().join("check-target"));
    let done: Output = bb_proc::run(command, "cargo check", TEST_DEADLINE, None).unwrap();
    let found: Vec<Message> = messages::messages(&done.stdout, &root);
    let errors: Vec<&Message> = found
        .iter()
        .filter(|m: &&Message| m.level == Level::Error)
        .collect();
    assert_eq!(errors.len(), 1, "{found:?}");
    let error: &Message = errors.first().unwrap();
    assert_eq!(
        (error.path.clone(), error.line),
        (root.join("beta/src/lib.rs"), BROKEN_LINE)
    );
    assert_ne!(done.code, Some(0));
}

#[test]
fn list_then_run_one_and_all_give_pass_fail_and_ignored() {
    let (base, root): (tempfile::TempDir, PathBuf) = workspace(Shape::Clean);
    let alpha: TestTarget = target(&base, &root);
    let list: SpecList = cargo_test::list(&alpha, &root.join("alpha/Cargo.toml")).unwrap();
    let mut names: Vec<String> = list.specs.iter().map(Fqn::to_string).collect();
    names.sort();
    let expected: Vec<String> = TESTS
        .iter()
        .map(|t: &&str| format!("alpha.{}", t.replace("::", ".")))
        .collect();
    assert_eq!(names, expected);
    let one: Selection = Selection {
        specs: vec![Fqn::new("alpha.tests.passes").unwrap()],
        category: None,
    };
    let single: RunReport = cargo_test::run(&alpha, &one, &StopFlag::new()).unwrap();
    assert_eq!(single.leaves.len(), 1, "{:?}", single.leaves);
    assert_eq!(
        outcome(&single, "alpha.tests.passes"),
        Some(Outcome::Passed)
    );
    let all: RunReport = cargo_test::run(&alpha, &Selection::default(), &StopFlag::new()).unwrap();
    assert_eq!(
        (
            outcome(&all, "alpha.tests.passes"),
            outcome(&all, "alpha.tests.fails"),
            outcome(&all, "alpha.tests.ignored"),
        ),
        (
            Some(Outcome::Passed),
            Some(Outcome::Failed),
            Some(Outcome::NotExecuted)
        )
    );
    let failed_message: Option<String> = all.leaves.iter().find_map(|leaf: &Leaf| match leaf {
        Leaf::Ran(result) if result.outcome == Outcome::Failed => result.message.clone(),
        Leaf::Ran(_) | Leaf::Missing(_) => None,
    });
    assert!(failed_message
        .unwrap_or_default()
        .contains("total is not four"));
    assert!(
        !root.join("target").exists(),
        "tests build in the scratch folder only"
    );
}
