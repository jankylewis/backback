//! What a list and a run give back.

use std::path::PathBuf;
use std::time::Duration;

use super::Fqn;

/// A TRX `outcome` value backback understands. Any other value is an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome {
    /// `Passed`.
    Passed,
    /// `Failed`.
    Failed,
    /// `NotExecuted` (ignored, skipped; NUnit `Inconclusive` comes out so).
    NotExecuted,
    /// `Inconclusive`.
    Inconclusive,
}

/// How a leaf shows: the Testkit maps four outcomes onto three verdicts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Verdict {
    /// The spec passed.
    Pass,
    /// The spec failed.
    Fail,
    /// The spec did not run to a verdict.
    Skip,
}

impl Outcome {
    /// The outcome for a TRX `outcome` text; `None` when unknown.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "Passed" => Some(Self::Passed),
            "Failed" => Some(Self::Failed),
            "NotExecuted" => Some(Self::NotExecuted),
            "Inconclusive" => Some(Self::Inconclusive),
            _ => None,
        }
    }

    /// The verdict the Testkit shows for this outcome.
    #[must_use]
    pub fn verdict(self) -> Verdict {
        match self {
            Self::Passed => Verdict::Pass,
            Self::Failed => Verdict::Fail,
            Self::NotExecuted | Self::Inconclusive => Verdict::Skip,
        }
    }
}

/// One spec's result, read from TRX.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecResult {
    /// `className + "." + name` of its test definition.
    pub fqn: Fqn,
    /// The outcome.
    pub outcome: Outcome,
    /// How long it ran (zero when TRX gives no duration).
    pub duration: Duration,
    /// The error message, if any.
    pub message: Option<String>,
    /// The stack trace, if any.
    pub stack: Option<String>,
}

/// One selected leaf after a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Leaf {
    /// TRX has a result for it.
    Ran(SpecResult),
    /// It was selected but TRX has no result for it: an error for this leaf.
    Missing(Fqn),
}

impl Leaf {
    /// The leaf's name.
    #[must_use]
    pub fn fqn(&self) -> &Fqn {
        match self {
            Self::Ran(result) => &result.fqn,
            Self::Missing(fqn) => fqn,
        }
    }
}

/// A finished run of one project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunReport {
    /// Every leaf: the selected ones in order, then any extra TRX results.
    pub leaves: Vec<Leaf>,
    /// The runner's exit code (`None` when killed by a signal).
    pub exit_code: Option<i32>,
    /// Wall time of the runner process.
    pub elapsed: Duration,
    /// Runner output from this run, kept in memory for the UI.
    pub stdout: String,
    /// Runner error output from this run, kept in memory for the UI.
    pub stderr: String,
}

impl RunReport {
    /// How many leaves have each verdict, and how many are missing:
    /// `(pass, fail, skip, missing)`.
    #[must_use]
    pub fn counts(&self) -> (usize, usize, usize, usize) {
        let mut counts: (usize, usize, usize, usize) = (0, 0, 0, 0);
        for leaf in &self.leaves {
            let slot: &mut usize = match leaf {
                Leaf::Ran(r) => match r.outcome.verdict() {
                    Verdict::Pass => &mut counts.0,
                    Verdict::Fail => &mut counts.1,
                    Verdict::Skip => &mut counts.2,
                },
                Leaf::Missing(_) => &mut counts.3,
            };
            *slot = slot.saturating_add(1);
        }
        counts
    }
}

/// The specs of one project, as discovery found them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecList {
    /// The project name.
    pub project: String,
    /// The test assembly.
    pub assembly: PathBuf,
    /// The specs, in discovery order.
    pub specs: Vec<Fqn>,
    /// Wall time of discovery.
    pub elapsed: Duration,
}
