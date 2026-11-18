//! Python tooling for backback (BBEOD28): where pyright is and how it
//! starts, which interpreter a folder uses, the Build menu's check
//! (pyright, or a syntax check when pyright is missing) and pytest.
//! Every process call has a deadline and its own group (the runner of
//! `bb_cargo`), and every Python child runs with
//! `PYTHONDONTWRITEBYTECODE=1`: nothing is written into the project.
//! There is no Python debugger, ever (headmate).

#[cfg(feature = "fixtures")]
pub mod fixtures;
pub mod interpreter;
#[path = "bb_pyright.rs"]
pub mod pyright;
pub mod run;

use std::path::PathBuf;
use std::process::Command;

pub use interpreter::{interpreter_for, Interpreter, InterpreterSource};
pub use run::bb_pytest::{TestRoot, PYTEST_DEADLINE};
pub use run::check::{Issue, Level, CHECK_DEADLINE};

/// Everything this crate can fail with.
#[derive(Debug, thiserror::Error)]
pub enum PythonError {
    /// A process failed to start, passed its deadline or was stopped.
    #[error(transparent)]
    Process(#[from] bb_cargo::CargoError),
    /// Output this crate cannot read.
    #[error("unreadable output of {what}: {detail}")]
    Unreadable {
        /// The tool.
        what: String,
        /// Why.
        detail: String,
    },
    /// No pyright that runs was found.
    #[error("{}", pyright::MISSING_HINT)]
    PyrightMissing,
    /// The pyright named by the override does not run.
    #[error("pyright at {} does not run: {reason}", binary.display())]
    PyrightBroken {
        /// The binary.
        binary: PathBuf,
        /// What it said.
        reason: String,
    },
    /// The interpreter has no pytest; the text is the install hint.
    #[error("{0}")]
    PytestMissing(String),
    /// A folder has no pytest root to run from.
    #[error("not a pytest root: {}", .0.display())]
    NoRoot(PathBuf),
    /// A file could not be read or written.
    #[error("{what}: {source}")]
    Io {
        /// What was done.
        what: String,
        /// The cause.
        #[source]
        source: std::io::Error,
    },
}

/// A command for a Python-side tool: it writes no bytecode
/// (`__pycache__`) into the project.
#[must_use]
pub fn python_command(program: &std::path::Path) -> Command {
    let mut command: Command = Command::new(program);
    command.env("PYTHONDONTWRITEBYTECODE", "1");
    command
}
