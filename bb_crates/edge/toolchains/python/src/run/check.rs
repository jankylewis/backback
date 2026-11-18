//! Build > Check for Python: pyright's diagnostics (`pyright --outputjson`)
//! with the folder's interpreter; when pyright is missing, a syntax check
//! of every `.py` file by the interpreter itself. Both list `file:line`.
//!
//! The syntax check calls `compile()` on each file's text instead of
//! `python -m py_compile`: `py_compile` writes a `.pyc` into the
//! project's `__pycache__` whatever `PYTHONDONTWRITEBYTECODE` says, and
//! backback never writes into the user's project (headmate, BBEOD28).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use bb_cargo::run::bb_proc::{self, Output};
use serde::Deserialize;

use crate::{python_command, PythonError};

/// How long a check may take.
pub const CHECK_DEADLINE: Duration = Duration::from_secs(600);

/// How bad an issue is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    /// An error.
    Error,
    /// A warning.
    Warning,
}

/// One issue at its place (1-based line and column).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Issue {
    /// The file, absolute.
    pub path: PathBuf,
    /// 1-based line.
    pub line: u32,
    /// 1-based column.
    pub column: u32,
    /// Error or warning.
    pub level: Level,
    /// The text.
    pub text: String,
}

/// Prints one JSON line per file that does not compile; skips dot
/// folders, `venv`, `__pycache__`, `node_modules` and `site-packages`.
pub const SYNTAX_SCRIPT: &str = r"import sys, os, json
root = sys.argv[1]
skip = {'__pycache__', 'node_modules', 'venv', 'site-packages'}
for d, dirs, files in os.walk(root):
    dirs[:] = sorted(x for x in dirs if not x.startswith('.') and x not in skip)
    for f in sorted(files):
        if not f.endswith('.py'):
            continue
        p = os.path.join(d, f)
        try:
            with open(p, 'rb') as h:
                compile(h.read(), p, 'exec', dont_inherit=True)
        except SyntaxError as e:
            print(json.dumps({'file': p, 'line': e.lineno or 1, 'column': e.offset or 1, 'message': e.msg}))
        except (ValueError, OSError) as e:
            print(json.dumps({'file': p, 'line': 1, 'column': 1, 'message': str(e)}))
";

/// The syntax check of `root` by `interpreter` (`-I -B`: isolated, no
/// bytecode).
#[must_use]
pub fn syntax_command(interpreter: &Path, root: &Path) -> Command {
    let mut command: Command = python_command(interpreter);
    command
        .args(["-I", "-B", "-c", SYNTAX_SCRIPT])
        .arg(root)
        .current_dir(root);
    command
}

#[derive(Deserialize)]
struct SyntaxLine {
    file: PathBuf,
    line: u32,
    column: u32,
    message: String,
}

/// The issues in the syntax check's output.
#[must_use]
pub fn syntax_issues(stdout: &str) -> Vec<Issue> {
    stdout
        .lines()
        .filter_map(|line: &str| serde_json::from_str::<SyntaxLine>(line).ok())
        .map(|found: SyntaxLine| Issue {
            path: found.file,
            line: found.line.max(1),
            column: found.column.max(1),
            level: Level::Error,
            text: found.message,
        })
        .collect()
}

/// `pyright --outputjson` on `root` with `interpreter`.
#[must_use]
pub fn pyright_command(cli: &Path, interpreter: &Path, root: &Path) -> Command {
    let mut command: Command = python_command(cli);
    command
        .arg("--outputjson")
        .arg("--pythonpath")
        .arg(interpreter)
        .current_dir(root);
    command
}

#[derive(Deserialize)]
struct Report {
    #[serde(rename = "generalDiagnostics", default)]
    diagnostics: Vec<Reported>,
}

#[derive(Deserialize)]
struct Reported {
    file: PathBuf,
    severity: String,
    message: String,
    range: Option<Range>,
}

#[derive(Deserialize)]
struct Range {
    start: Point,
}

#[derive(Deserialize)]
struct Point {
    line: u32,
    character: u32,
}

/// The errors and warnings in `pyright --outputjson` output (its
/// information lines are left out).
///
/// # Errors
/// [`PythonError::Unreadable`] when the output is not pyright's JSON.
pub fn pyright_issues(stdout: &str) -> Result<Vec<Issue>, PythonError> {
    let report: Report = serde_json::from_str(stdout).map_err(|error: serde_json::Error| {
        PythonError::Unreadable {
            what: String::from("pyright --outputjson"),
            detail: error.to_string(),
        }
    })?;
    Ok(report
        .diagnostics
        .into_iter()
        .filter_map(|found: Reported| {
            let level: Level = match found.severity.as_str() {
                "error" => Level::Error,
                "warning" => Level::Warning,
                _ => return None,
            };
            let (line, character): (u32, u32) = found
                .range
                .map_or((0, 0), |r: Range| (r.start.line, r.start.character));
            Some(Issue {
                path: found.file,
                line: line.saturating_add(1),
                column: character.saturating_add(1),
                level,
                text: found.message,
            })
        })
        .collect())
}

/// Run the syntax check of `root` and return its issues.
///
/// # Errors
/// The interpreter cannot start, passes [`CHECK_DEADLINE`], or fails.
pub fn syntax(interpreter: &Path, root: &Path) -> Result<Vec<Issue>, PythonError> {
    let done: Output = bb_proc::run(
        syntax_command(interpreter, root),
        "python syntax check",
        CHECK_DEADLINE,
        None,
    )?;
    if done.code != Some(0) {
        return Err(
            bb_cargo::CargoError::failed("python syntax check", done.code, &done.stderr).into(),
        );
    }
    Ok(syntax_issues(&done.stdout))
}

/// Run pyright on `root` with `interpreter` and return its issues.
///
/// # Errors
/// pyright cannot start, passes [`CHECK_DEADLINE`], fails (any exit but
/// 0 or 1, "errors found"), or prints no JSON.
pub fn pyright(cli: &Path, interpreter: &Path, root: &Path) -> Result<Vec<Issue>, PythonError> {
    let done: Output = bb_proc::run(
        pyright_command(cli, interpreter, root),
        "pyright",
        CHECK_DEADLINE,
        None,
    )?;
    if !matches!(done.code, Some(0 | 1)) {
        return Err(bb_cargo::CargoError::failed("pyright", done.code, &done.stderr).into());
    }
    pyright_issues(&done.stdout)
}
