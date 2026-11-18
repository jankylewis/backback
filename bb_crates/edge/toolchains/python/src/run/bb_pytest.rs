//! pytest: list a folder's tests (`--collect-only -q`) and run one, a
//! file's or all of them. Results come from a junit report written into
//! backback's scratch folder, never the project; `-p no:cacheprovider`
//! keeps `.pytest_cache` out and every child writes no bytecode.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use bb_cargo::run::bb_proc::{self, Output};
use bb_testkit::{Fqn, Leaf, RunReport, Selection, SpecList, SpecResult, StopFlag};

use super::junit;
use crate::{python_command, PythonError};

/// How long a list or a run may take.
pub const PYTEST_DEADLINE: Duration = Duration::from_secs(1800);
/// The files that make a folder a pytest root, in the order looked for.
pub const ROOT_FILES: [&str; 5] = [
    "pyproject.toml",
    "pytest.ini",
    "setup.cfg",
    "setup.py",
    "tox.ini",
];
/// The text pytest's absence prints.
const NO_MODULE: &str = "No module named pytest";

/// One pytest root: where pytest runs, with which Python, and where its
/// reports go.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestRoot {
    /// The folder pytest runs in (the root file's folder).
    pub dir: PathBuf,
    /// The interpreter (`… -m pytest`).
    pub interpreter: PathBuf,
    /// backback's scratch folder for junit reports, never inside `dir`.
    pub scratch: PathBuf,
}

/// The root file of the pytest root at `dir`, if it is one.
#[must_use]
pub fn root_file(dir: &Path) -> Option<PathBuf> {
    ROOT_FILES
        .iter()
        .map(|name: &&str| dir.join(name))
        .find(|file: &PathBuf| file.is_file())
}

/// What the status says when `interpreter` has no pytest.
#[must_use]
pub fn missing_hint(interpreter: &Path) -> String {
    let shown: String = interpreter.display().to_string();
    format!("pytest not installed for {shown}: {shown} -m pip install pytest")
}

impl TestRoot {
    fn command(&self, args: &[String]) -> Command {
        let mut command: Command = python_command(&self.interpreter);
        command
            .args(["-m", "pytest", "-p", "no:cacheprovider", "--color=no"])
            .args(args)
            .current_dir(&self.dir);
        command
    }

    fn checked(&self, done: Output) -> Result<Output, PythonError> {
        if done.stderr.contains(NO_MODULE) || done.stdout.contains(NO_MODULE) {
            return Err(PythonError::PytestMissing(missing_hint(&self.interpreter)));
        }
        Ok(done)
    }
}

/// The tests of `root`, by `--collect-only -q`.
///
/// # Errors
/// pytest is missing ([`PythonError::PytestMissing`]), fails to run,
/// passes its deadline, or fails to collect.
pub fn list(root: &TestRoot, file: &Path) -> Result<SpecList, PythonError> {
    let args: [String; 2] = [String::from("--collect-only"), String::from("-q")];
    let done: Output = bb_proc::run(
        root.command(&args),
        "pytest --collect-only",
        PYTEST_DEADLINE,
        None,
    )?;
    let done: Output = root.checked(done)?;
    // 5: no tests collected, an empty list.
    if !matches!(done.code, Some(0 | 5)) {
        return Err(bb_cargo::CargoError::failed(
            "pytest --collect-only",
            done.code,
            &failure_text(&done),
        )
        .into());
    }
    Ok(SpecList {
        project: project_name(&root.dir),
        assembly: file.to_path_buf(),
        specs: collected(&done.stdout),
        elapsed: done.elapsed,
    })
}

/// The test names in `--collect-only -q` output.
#[must_use]
pub fn collected(stdout: &str) -> Vec<Fqn> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|line: &&str| line.contains("::"))
        .filter_map(junit::fqn_of)
        .collect()
}

/// Run `selection` of `root`: every test when it names none.
///
/// # Errors
/// pytest is missing, fails to start, passes its deadline, `stop` is set,
/// or leaves no readable report.
pub fn run(
    root: &TestRoot,
    selection: &Selection,
    stop: &StopFlag,
) -> Result<RunReport, PythonError> {
    std::fs::create_dir_all(&root.scratch).map_err(|source: std::io::Error| PythonError::Io {
        what: format!("create {}", root.scratch.display()),
        source,
    })?;
    let report: PathBuf = root
        .scratch
        .join(format!("pytest-{}.xml", std::process::id()));
    let _stale: std::io::Result<()> = std::fs::remove_file(&report);
    let mut args: Vec<String> = vec![
        String::from("-q"),
        format!("--junitxml={}", report.display()),
    ];
    args.extend(
        selection
            .specs
            .iter()
            .filter_map(|fqn: &Fqn| junit::node_id_of(&root.dir, fqn)),
    );
    let done: Output = bb_proc::run(root.command(&args), "pytest", PYTEST_DEADLINE, Some(stop))?;
    let done: Output = root.checked(done)?;
    let xml: String =
        std::fs::read_to_string(&report).map_err(|source: std::io::Error| PythonError::Io {
            what: format!("pytest left no report ({})", failure_text(&done)),
            source,
        })?;
    let _removed: std::io::Result<()> = std::fs::remove_file(&report);
    Ok(RunReport {
        leaves: leaves(&selection.specs, junit::results(&xml)?),
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

/// The last lines pytest printed, from stderr or else stdout.
fn failure_text(done: &Output) -> String {
    if done.stderr.trim().is_empty() {
        done.stdout.clone()
    } else {
        done.stderr.clone()
    }
}

fn project_name(dir: &Path) -> String {
    dir.file_name()
        .map(|n: &std::ffi::OsStr| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::process::Command;

    use super::TestRoot;
    use crate::run::check;

    fn env_of(command: &Command, key: &str) -> Option<String> {
        command
            .get_envs()
            .find(|(k, _): &(&std::ffi::OsStr, Option<&std::ffi::OsStr>)| *k == key)
            .and_then(|(_, v): (&std::ffi::OsStr, Option<&std::ffi::OsStr>)| v)
            .map(|v: &std::ffi::OsStr| v.to_string_lossy().into_owned())
    }

    fn args_of(command: &Command) -> Vec<String> {
        command
            .get_args()
            .map(|a: &std::ffi::OsStr| a.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn every_python_child_writes_no_bytecode_and_pytest_keeps_no_cache() {
        let root: TestRoot = TestRoot {
            dir: PathBuf::from("/w"),
            interpreter: PathBuf::from("/usr/bin/python3"),
            scratch: PathBuf::from("/tmp/s"),
        };
        let pytest: Command = root.command(&[]);
        let children: [Command; 3] = [
            root.command(&[]),
            check::syntax_command(&root.interpreter, &root.dir),
            check::pyright_command(&PathBuf::from("/bin/pyright"), &root.interpreter, &root.dir),
        ];
        for child in &children {
            assert_eq!(
                env_of(child, "PYTHONDONTWRITEBYTECODE").as_deref(),
                Some("1")
            );
        }
        let args: Vec<String> = args_of(&pytest);
        assert!(
            args.windows(2)
                .any(|w: &[String]| w == ["-p", "no:cacheprovider"]),
            "{args:?}"
        );
    }
}
