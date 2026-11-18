//! Rust tooling for backback (BBEOD27): where rust-analyzer is and how it
//! starts, the Cargo workspace's members, `cargo build`/`check` messages
//! and `cargo test`. Every process call has a deadline and its own group.
//! There is no Rust debugger, ever (headmate).

#[path = "bb_analyzer.rs"]
pub mod analyzer;
#[cfg(feature = "fixtures")]
pub mod fixtures;
pub mod project;
pub mod run;

use std::path::PathBuf;
use std::time::Duration;

pub use project::bb_metadata::{members, CargoWorkspace, Member, METADATA_DEADLINE};
pub use project::messages::{BuildKind, Level, Message};
pub use run::cargo_test::{TestTarget, TEST_DEADLINE};

/// Everything this crate can fail with.
#[derive(Debug, thiserror::Error)]
pub enum CargoError {
    /// A tool could not be started.
    #[error("cannot start {what}: {source}")]
    Spawn {
        /// The tool.
        what: String,
        /// The cause.
        #[source]
        source: std::io::Error,
    },
    /// A tool passed its deadline and was ended.
    #[error("{what} timed out after {} s", after.as_secs())]
    Timeout {
        /// The tool.
        what: String,
        /// The deadline.
        after: Duration,
    },
    /// The caller stopped the run.
    #[error("stopped")]
    Stopped,
    /// A tool that must succeed exited non-zero.
    #[error("{what} exited with {code:?}: {detail}")]
    Failed {
        /// The tool.
        what: String,
        /// The exit code (`None`: ended by a signal).
        code: Option<i32>,
        /// The last lines it printed on stderr.
        detail: String,
    },
    /// Output this crate cannot read.
    #[error("unreadable cargo output: {0}")]
    Json(#[from] serde_json::Error),
    /// No rust-analyzer that runs was found.
    #[error("{}", analyzer::MISSING_HINT)]
    AnalyzerMissing,
    /// The rust-analyzer named by the override does not run.
    #[error("rust-analyzer at {} does not run: {reason}", binary.display())]
    AnalyzerBroken {
        /// The binary.
        binary: PathBuf,
        /// What it said.
        reason: String,
    },
}

impl CargoError {
    /// A non-zero exit, with the last lines of `stderr`.
    #[must_use]
    pub fn failed(what: &str, code: Option<i32>, stderr: &str) -> Self {
        let lines: Vec<&str> = stderr.lines().collect();
        let tail: &[&str] = lines
            .get(lines.len().saturating_sub(5)..)
            .unwrap_or_default();
        Self::Failed {
            what: what.to_owned(),
            code,
            detail: tail.join("\n"),
        }
    }
}
