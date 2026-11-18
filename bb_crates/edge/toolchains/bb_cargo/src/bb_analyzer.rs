//! rust-analyzer: where it is, and how it starts without writing into the
//! user's project (its `cargo check` builds into backback's scratch).
//!
//! Discovery order (headmate, BBEOD27): [`ENV_OVERRIDE`], then `PATH`, then
//! `rustup which rust-analyzer`. Each candidate must answer `--version`:
//! the rustup shim on `PATH` exists without the component and only prints
//! an error, so a file that exists is not a server that runs.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use serde_json::{json, Value};

use crate::run::bb_proc::{self, Output};
use crate::CargoError;

/// The binary name.
pub const BINARY: &str = "rust-analyzer";
/// Environment variable that names the server to use.
pub const ENV_OVERRIDE: &str = "BACKBACK_RUST_ANALYZER";
/// What the status bar says when no candidate runs.
pub const MISSING_HINT: &str = "rust-analyzer not installed: rustup component add rust-analyzer";

/// How long one `--version` probe may take.
const PROBE: Duration = Duration::from_secs(10);

/// Where the server was found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// [`ENV_OVERRIDE`].
    Env,
    /// A `rust-analyzer` on `PATH`.
    Path,
    /// `rustup which rust-analyzer`.
    Rustup,
}

/// A server that answered `--version`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    /// The binary.
    pub binary: PathBuf,
    /// Where it came from.
    pub source: Source,
    /// Its `--version` line.
    pub version: String,
}

/// What discovery reads, so tests can give their own.
#[derive(Clone, Debug, Default)]
pub struct Inputs {
    /// The value of [`ENV_OVERRIDE`].
    pub env: Option<OsString>,
    /// The value of `PATH`.
    pub path: Option<OsString>,
    /// Whether `rustup which` may be asked.
    pub rustup: bool,
}

impl Inputs {
    /// This process's environment.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            env: std::env::var_os(ENV_OVERRIDE),
            path: std::env::var_os("PATH"),
            rustup: true,
        }
    }
}

/// Find a rust-analyzer that runs.
///
/// # Errors
/// [`CargoError::AnalyzerBroken`] when [`ENV_OVERRIDE`] names one that does
/// not run (a set override is never skipped silently);
/// [`CargoError::AnalyzerMissing`] when no candidate runs.
pub fn locate(inputs: &Inputs) -> Result<Found, CargoError> {
    if let Some(named) = inputs.env.as_ref().filter(|v: &&OsString| !v.is_empty()) {
        let binary: PathBuf = PathBuf::from(named);
        return probe(&binary).map(|version: String| Found {
            binary: binary.clone(),
            source: Source::Env,
            version,
        });
    }
    let on_path: Option<PathBuf> = inputs.path.as_ref().and_then(|p: &OsString| on_path(p));
    if let Some(binary) = on_path {
        if let Ok(version) = probe(&binary) {
            return Ok(Found {
                binary,
                source: Source::Path,
                version,
            });
        }
    }
    let from_rustup: Option<PathBuf> = inputs.rustup.then(rustup_which).flatten();
    if let Some(binary) = from_rustup {
        if let Ok(version) = probe(&binary) {
            return Ok(Found {
                binary,
                source: Source::Rustup,
                version,
            });
        }
    }
    Err(CargoError::AnalyzerMissing)
}

fn on_path(path: &OsString) -> Option<PathBuf> {
    std::env::split_paths(path)
        .map(|dir: PathBuf| dir.join(BINARY))
        .find(|candidate: &PathBuf| candidate.is_file())
}

fn rustup_which() -> Option<PathBuf> {
    let mut command: Command = Command::new("rustup");
    command.args(["which", BINARY]);
    let done: Output = bb_proc::run(command, "rustup which", PROBE, None).ok()?;
    let line: &str = done.stdout.lines().next()?.trim();
    (done.code == Some(0) && !line.is_empty()).then(|| PathBuf::from(line))
}

/// The `--version` line of `binary`, when it runs as rust-analyzer.
fn probe(binary: &Path) -> Result<String, CargoError> {
    let mut command: Command = Command::new(binary);
    command.arg("--version");
    let done: Output = bb_proc::run(command, BINARY, PROBE, None)
        .map_err(|error: CargoError| broken(binary, error.to_string()))?;
    let line: String = done
        .stdout
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_owned();
    if done.code == Some(0) && line.starts_with(BINARY) {
        Ok(line)
    } else {
        let said: &str = done.stderr.lines().next().unwrap_or_default().trim();
        Err(broken(binary, format!("exit {:?}: {said}", done.code)))
    }
}

fn broken(binary: &Path, reason: String) -> CargoError {
    CargoError::AnalyzerBroken {
        binary: binary.to_path_buf(),
        reason,
    }
}

/// The server's settings: its `cargo check` and build scripts build into
/// `target_dir`, never the project's `target/`. Given both as
/// `initializationOptions` and as the `rust-analyzer` configuration section.
#[must_use]
pub fn settings(target_dir: &Path) -> Value {
    json!({
        "cargo": { "targetDir": target_dir },
        "check": { "command": "check" },
        "checkOnSave": true
    })
}

/// The command that starts `binary` over stdio, with `CARGO_TARGET_DIR`
/// set to `target_dir` for it and every cargo it runs.
#[must_use]
pub fn command(binary: &Path, target_dir: &Path) -> Command {
    let mut command: Command = Command::new(binary);
    command.env("CARGO_TARGET_DIR", target_dir);
    command
}
