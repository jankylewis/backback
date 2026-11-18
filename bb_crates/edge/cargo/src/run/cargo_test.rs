//! `cargo test`: list a member's tests (`-- --list`) and run one, some or
//! all of them, read from libtest's plain output. A test `tests::passes`
//! of package `alpha` is the Tests panel's `alpha.tests.passes`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use bb_testkit::{Fqn, Leaf, Outcome, RunReport, Selection, SpecList, SpecResult, StopFlag};

use super::bb_proc::{self, Output};
use crate::CargoError;

/// How long a list (it builds the tests) or a run may take.
pub const TEST_DEADLINE: Duration = Duration::from_secs(1800);

/// One member's tests: where cargo runs, which package, where it builds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestTarget {
    /// The workspace root (cargo's folder).
    pub root: PathBuf,
    /// The package name.
    pub package: String,
    /// The build folder: backback's scratch, never the project's `target/`.
    pub target_dir: PathBuf,
}

impl TestTarget {
    fn command(&self, after: &[String]) -> Command {
        let mut command: Command = Command::new("cargo");
        command
            .args(["test", "-p", &self.package, "--color", "never", "--"])
            .args(after)
            .current_dir(&self.root)
            .env("CARGO_TARGET_DIR", &self.target_dir);
        command
    }
}

/// The panel's name for `rust_name` of `package`; `None` for a doc test.
#[must_use]
pub fn fqn_of(package: &str, rust_name: &str) -> Option<Fqn> {
    if rust_name.contains(" - ") {
        return None;
    }
    Fqn::new(&format!("{package}.{}", rust_name.replace("::", "."))).ok()
}

/// libtest's name for `fqn` of `package`; `None` when it is not one of its.
#[must_use]
pub fn rust_name_of(package: &str, fqn: &Fqn) -> Option<String> {
    let rest: &str = fqn.as_str().strip_prefix(package)?.strip_prefix('.')?;
    Some(rest.replace('.', "::"))
}

/// List the tests of `target` (builds them into its scratch folder).
///
/// # Errors
/// The tool fails to run, passes its deadline, or exits non-zero.
pub fn list(target: &TestTarget, manifest: &Path) -> Result<SpecList, CargoError> {
    let done: Output = bb_proc::run(
        target.command(&[String::from("--list")]),
        "cargo test --list",
        TEST_DEADLINE,
        None,
    )?;
    if done.code != Some(0) {
        return Err(CargoError::failed(
            "cargo test --list",
            done.code,
            &done.stderr,
        ));
    }
    Ok(SpecList {
        project: target.package.clone(),
        assembly: manifest.to_path_buf(),
        specs: listed(&target.package, &done.stdout),
        elapsed: done.elapsed,
    })
}

/// The test names in `cargo test -- --list` output.
#[must_use]
pub fn listed(package: &str, stdout: &str) -> Vec<Fqn> {
    stdout
        .lines()
        .filter_map(|line: &str| line.strip_suffix(": test"))
        .filter_map(|name: &str| fqn_of(package, name))
        .collect()
}

/// Run `selection` of `target`: every test when it names none.
///
/// # Errors
/// The tool fails to run, passes its deadline, or `stop` is set.
pub fn run(
    target: &TestTarget,
    selection: &Selection,
    stop: &StopFlag,
) -> Result<RunReport, CargoError> {
    let mut after: Vec<String> = Vec::new();
    if !selection.specs.is_empty() {
        after.push(String::from("--exact"));
        after.extend(
            selection
                .specs
                .iter()
                .filter_map(|fqn: &Fqn| rust_name_of(&target.package, fqn)),
        );
    }
    let done: Output = bb_proc::run(
        target.command(&after),
        "cargo test",
        TEST_DEADLINE,
        Some(stop),
    )?;
    let ran: Vec<SpecResult> = results(&target.package, &done.stdout);
    Ok(RunReport {
        leaves: leaves(&selection.specs, ran),
        exit_code: done.code,
        elapsed: done.elapsed,
        stdout: done.stdout,
        stderr: done.stderr,
    })
}

fn leaves(selected: &[Fqn], ran: Vec<SpecResult>) -> Vec<Leaf> {
    let mut out: Vec<Leaf> = selected
        .iter()
        .filter(|fqn: &&Fqn| !ran.iter().any(|r: &SpecResult| &r.fqn == *fqn))
        .cloned()
        .map(Leaf::Missing)
        .collect();
    out.extend(ran.into_iter().map(Leaf::Ran));
    out
}

/// Every `test NAME ... ok|FAILED|ignored` line, with each failure's
/// captured output as its message.
#[must_use]
pub fn results(package: &str, stdout: &str) -> Vec<SpecResult> {
    let mut found: Vec<SpecResult> = Vec::new();
    for line in stdout.lines() {
        let Some((name, verdict)): Option<(&str, &str)> = line
            .strip_prefix("test ")
            .and_then(|rest: &str| rest.rsplit_once(" ... "))
        else {
            continue;
        };
        let outcome: Outcome = match verdict.trim() {
            "ok" => Outcome::Passed,
            "FAILED" => Outcome::Failed,
            other if other.starts_with("ignored") => Outcome::NotExecuted,
            _ => continue,
        };
        let Some(fqn): Option<Fqn> = fqn_of(package, name) else {
            continue;
        };
        let message: Option<String> = (outcome == Outcome::Failed)
            .then(|| captured(stdout, name))
            .flatten();
        found.push(SpecResult {
            fqn,
            outcome,
            duration: Duration::ZERO,
            message,
            stack: None,
        });
    }
    found
}

/// The `---- NAME stdout ----` block of a failed test.
fn captured(stdout: &str, name: &str) -> Option<String> {
    let head: String = format!("---- {name} stdout ----");
    let (_, after): (&str, &str) = stdout.split_once(&head)?;
    let body: Vec<&str> = after
        .lines()
        .skip(1)
        .take_while(|line: &&str| !line.starts_with("---- ") && *line != "failures:")
        .collect();
    let text: String = body.join("\n").trim().to_owned();
    (!text.is_empty()).then_some(text)
}
