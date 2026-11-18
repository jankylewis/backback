//! pyright: where it is, and how it starts on a folder with the chosen
//! interpreter.
//!
//! Discovery order (headmate, BBEOD28): [`ENV_OVERRIDE`], then
//! `pyright-langserver` on `PATH`, then `basedpyright-langserver`. Each
//! candidate's command-line sibling (`pyright`, `basedpyright`, in the same
//! folder) must answer `--version`: a server file alone is not a server
//! that runs, and the sibling is what Build > Check runs.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use bb_cargo::run::bb_proc::{self, Output};
use serde_json::{json, Value};

use crate::{python_command, PythonError};

/// The language servers looked for on `PATH`, in order.
pub const SERVERS: [&str; 2] = ["pyright-langserver", "basedpyright-langserver"];
/// Environment variable that names the language server to use.
pub const ENV_OVERRIDE: &str = "BACKBACK_PYRIGHT";
/// What the status bar says when no candidate runs.
pub const MISSING_HINT: &str = "pyright not installed: npm install -g pyright";

/// How long one `--version` probe may take.
const PROBE: Duration = Duration::from_secs(10);
/// The suffix that turns a server's name into its command line's.
const SERVER_SUFFIX: &str = "-langserver";

/// Where the server was found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// [`ENV_OVERRIDE`].
    Env,
    /// On `PATH`.
    Path,
}

/// A pyright that answered `--version`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    /// The language server (`… --stdio`).
    pub server: PathBuf,
    /// Its command line, for Build > Check.
    pub cli: PathBuf,
    /// Where it came from.
    pub source: Source,
    /// The command line's `--version` line.
    pub version: String,
}

/// What discovery reads, so tests can give their own.
#[derive(Clone, Debug, Default)]
pub struct Inputs {
    /// The value of [`ENV_OVERRIDE`].
    pub env: Option<OsString>,
    /// The value of `PATH`.
    pub path: Option<OsString>,
}

impl Inputs {
    /// This process's environment.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            env: std::env::var_os(ENV_OVERRIDE),
            path: std::env::var_os("PATH"),
        }
    }
}

/// Find a pyright that runs.
///
/// # Errors
/// [`PythonError::PyrightBroken`] when [`ENV_OVERRIDE`] names one that does
/// not run (a set override is never skipped silently);
/// [`PythonError::PyrightMissing`] when no candidate runs.
pub fn locate(inputs: &Inputs) -> Result<Found, PythonError> {
    if let Some(named) = inputs.env.as_ref().filter(|v: &&OsString| !v.is_empty()) {
        return found(PathBuf::from(named), Source::Env);
    }
    let dirs: Vec<PathBuf> = inputs
        .path
        .as_ref()
        .map(|p: &OsString| std::env::split_paths(p).collect())
        .unwrap_or_default();
    for name in SERVERS {
        let candidate: Option<PathBuf> = dirs
            .iter()
            .map(|dir: &PathBuf| dir.join(name))
            .find(|file: &PathBuf| file.is_file());
        if let Some(server) = candidate {
            if let Ok(done) = found(server, Source::Path) {
                return Ok(done);
            }
        }
    }
    Err(PythonError::PyrightMissing)
}

fn found(server: PathBuf, source: Source) -> Result<Found, PythonError> {
    let cli: PathBuf = cli_of(&server);
    let version: String = probe(&cli).map_err(|reason: String| PythonError::PyrightBroken {
        binary: server.clone(),
        reason,
    })?;
    Ok(Found {
        server,
        cli,
        source,
        version,
    })
}

/// The command line beside `server`: `pyright` for `pyright-langserver`.
#[must_use]
pub fn cli_of(server: &Path) -> PathBuf {
    let name: String = server
        .file_name()
        .map(|n: &std::ffi::OsStr| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let cli: &str = name.strip_suffix(SERVER_SUFFIX).unwrap_or(&name);
    server.with_file_name(cli)
}

fn probe(cli: &Path) -> Result<String, String> {
    let mut command: Command = python_command(cli);
    command.arg("--version");
    let done: Output = bb_proc::run(command, "pyright --version", PROBE, None)
        .map_err(|error: bb_cargo::CargoError| error.to_string())?;
    let line: String = done
        .stdout
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_owned();
    if done.code == Some(0) && line.contains("pyright") {
        Ok(line)
    } else {
        let said: &str = done.stderr.lines().next().unwrap_or_default().trim();
        Err(format!("exit {:?}: {said}", done.code))
    }
}

/// The `workspace/configuration` sections pyright asks for: the chosen
/// interpreter, and diagnostics for open files only.
#[must_use]
pub fn settings(interpreter: &Path) -> Vec<(String, Value)> {
    let analysis: Value = json!({ "diagnosticMode": "openFilesOnly", "autoSearchPaths": true });
    vec![
        (String::from("python"), json!({ "pythonPath": interpreter })),
        (String::from("python.analysis"), analysis.clone()),
        (String::from("basedpyright.analysis"), analysis),
    ]
}

/// The command that starts the server over stdio.
#[must_use]
pub fn command(server: &Path) -> Command {
    let mut command: Command = python_command(server);
    command.arg("--stdio");
    command
}
