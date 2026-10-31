//! The one trait both backends implement, and the run every caller uses:
//! build if asked, run into a fresh TRX folder, read the TRX, map leaves.

use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::time::Duration;

use bb_dotnet::ProjectPath;

use crate::backend::{Finished, TempDir};
use crate::model::{Category, Fqn, Leaf, RunReport, SpecList, SpecResult};
use crate::parse::parse_trx;
use crate::TestkitError;

/// Deadlines for each kind of step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Deadlines {
    /// Discovery of one project.
    pub list: Duration,
    /// `dotnet build` of one project.
    pub build: Duration,
    /// One run of one project.
    pub run: Duration,
}

impl Default for Deadlines {
    fn default() -> Self {
        Self {
            list: Duration::from_secs(120),
            build: Duration::from_secs(600),
            run: Duration::from_secs(1800),
        }
    }
}

/// What to run of one project.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Selection {
    /// These specs only; empty means every spec.
    pub specs: Vec<Fqn>,
    /// Only specs of this category.
    pub category: Option<Category>,
}

/// A way to list and run the tests of one project.
pub trait TestBackend {
    /// A short name for messages (`apex`, `vstest`, `mtp`).
    fn name(&self) -> &'static str;

    /// Discover the specs of `project` without building it.
    fn list(
        &self,
        project: &ProjectPath,
    ) -> impl Future<Output = Result<SpecList, TestkitError>> + Send;

    /// Whether a run builds the project first.
    fn builds_before_run(&self) -> bool;

    /// `dotnet build` the project in the backend's configuration.
    fn build(&self, project: &ProjectPath)
        -> impl Future<Output = Result<(), TestkitError>> + Send;

    /// Run `selection` of `project`, writing TRX to `trx`. A non-zero exit
    /// is not an error here: failed specs exit non-zero too.
    fn execute(
        &self,
        project: &ProjectPath,
        selection: &Selection,
        trx: &Path,
    ) -> impl Future<Output = Result<Finished, TestkitError>> + Send;
}

/// Run `selection` of `project` with `backend` and read the result.
///
/// The leaves are the selected specs; with no selection, every listed spec
/// (no listing when a category filters, since the list has no categories).
/// TRX results for other names are added after them.
///
/// # Errors
/// Build, list or run failures; [`TestkitError::NoTrx`] when the run wrote
/// no TRX (even with exit code 0); [`TestkitError::Trx`] when it is bad.
pub async fn run<B: TestBackend + Sync>(
    backend: &B,
    project: &ProjectPath,
    selection: &Selection,
) -> Result<RunReport, TestkitError> {
    if backend.builds_before_run() {
        backend.build(project).await?;
    }
    let expected: Vec<Fqn> = match (selection.specs.as_slice(), &selection.category) {
        ([], None) => backend.list(project).await?.specs,
        ([], Some(_)) => Vec::new(),
        (specs, _) => specs.to_vec(),
    };
    let folder: TempDir = TempDir::new()?;
    let trx: PathBuf = folder.path().join("run.trx");
    let done: Finished = backend.execute(project, selection, &trx).await?;
    let text: String = std::fs::read_to_string(&trx)
        .map_err(|_: std::io::Error| TestkitError::NoTrx(trx.clone()))?;
    let results: Vec<SpecResult> = parse_trx(&text)?;
    Ok(RunReport {
        leaves: leaves(&expected, results),
        exit_code: done.code,
        elapsed: done.elapsed,
    })
}

/// Each expected name takes the next unused result of that name, else it
/// is [`Leaf::Missing`]; unused results follow in TRX order.
#[must_use]
pub(crate) fn leaves(expected: &[Fqn], results: Vec<SpecResult>) -> Vec<Leaf> {
    let mut slots: Vec<Option<SpecResult>> = results.into_iter().map(Some).collect();
    let mut by_name: HashMap<Fqn, Vec<usize>> = HashMap::new();
    for (index, slot) in slots.iter().enumerate().rev() {
        if let Some(result) = slot {
            by_name.entry(result.fqn.clone()).or_default().push(index);
        }
    }
    let mut out: Vec<Leaf> = Vec::with_capacity(expected.len());
    for name in expected {
        let taken: Option<SpecResult> = by_name
            .get_mut(name)
            .and_then(Vec::pop)
            .and_then(|i: usize| slots.get_mut(i).and_then(Option::take));
        out.push(taken.map_or_else(|| Leaf::Missing(name.clone()), Leaf::Ran));
    }
    out.extend(slots.into_iter().flatten().map(Leaf::Ran));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Outcome;

    fn fqn(name: &str) -> Fqn {
        Fqn::new(name).unwrap()
    }

    fn ran(name: &str, outcome: Outcome) -> SpecResult {
        SpecResult {
            fqn: fqn(name),
            outcome,
            duration: Duration::ZERO,
            message: None,
            stack: None,
        }
    }

    #[test]
    fn a_selected_spec_absent_from_trx_is_missing_and_extra_results_follow() {
        let expected: Vec<Fqn> = vec![fqn("N.C.A"), fqn("N.C.B"), fqn("N.C.A")];
        let results: Vec<SpecResult> = vec![
            ran("N.C.X", Outcome::Passed),
            ran("N.C.A", Outcome::Failed),
            ran("N.C.A", Outcome::Passed),
        ];
        let got: Vec<Leaf> = leaves(&expected, results);
        assert_eq!(
            got,
            vec![
                Leaf::Ran(ran("N.C.A", Outcome::Failed)),
                Leaf::Missing(fqn("N.C.B")),
                Leaf::Ran(ran("N.C.A", Outcome::Passed)),
                Leaf::Ran(ran("N.C.X", Outcome::Passed)),
            ]
        );
    }

    #[test]
    fn counts_split_verdicts_and_missing() {
        let report: RunReport = RunReport {
            leaves: vec![
                Leaf::Ran(ran("N.C.A", Outcome::Passed)),
                Leaf::Ran(ran("N.C.B", Outcome::Failed)),
                Leaf::Ran(ran("N.C.C", Outcome::NotExecuted)),
                Leaf::Ran(ran("N.C.D", Outcome::Inconclusive)),
                Leaf::Missing(fqn("N.C.E")),
            ],
            exit_code: Some(1),
            elapsed: Duration::ZERO,
        };
        assert_eq!(report.counts(), (1, 1, 2, 1));
    }
}
