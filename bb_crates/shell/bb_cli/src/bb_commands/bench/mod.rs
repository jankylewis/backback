//! `bb bench`: the fixed benchmark set. Each item gets one warm-up and five
//! measured runs; one table and one JSON line per item report median and p95.

mod bb_items;
mod stats;

use std::path::{Path, PathBuf};

use serde::Serialize;

use super::CommandError;
use stats::Summary;

/// Measured runs per item, after one warm-up.
pub(crate) const RUNS: usize = 5;

/// The harness solution, relative to the repo root.
const HARNESS_SLNX: &str = "fixtures/_Harness/Harness.slnx";

/// Bench errors.
#[derive(Debug, thiserror::Error)]
pub enum BenchError {
    /// No backback repo at or above the current folder.
    #[error("no backback repo ({HARNESS_SLNX}) at or above {0}")]
    NoRepo(PathBuf),
    /// A file or process step failed.
    #[error("cannot {what}: {source}")]
    Io {
        /// The step.
        what: &'static str,
        /// Cause.
        #[source]
        source: std::io::Error,
    },
    /// `bb doctor` passed its deadline.
    #[error("bb doctor ran past {0:?}")]
    ColdStartTimeout(std::time::Duration),
    /// `bb doctor` exited non-zero.
    #[error("bb doctor failed: {0}")]
    ColdStartFailed(std::process::ExitStatus),
    /// The fixtures hold no `.cs` file.
    #[error("fixtures hold no .cs file")]
    NoCsFile,
    /// A row did not encode as JSON.
    #[error("cannot encode a row: {0}")]
    Json(#[from] serde_json::Error),
    /// An item produced no samples.
    #[error("no samples")]
    NoSamples,
}

/// What a row counts.
#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) enum Unit {
    /// Wall time in microseconds.
    #[serde(rename = "us")]
    Micros,
    /// Size in bytes.
    #[serde(rename = "bytes")]
    Bytes,
}

/// One result row (also one JSON line).
#[derive(Debug, Serialize)]
pub(crate) struct Row {
    item: &'static str,
    unit: Unit,
    median: u64,
    p95: u64,
    runs: usize,
    detail: String,
}

impl Row {
    fn new(item: &'static str, unit: Unit, summary: Summary, detail: String) -> Self {
        Self {
            item,
            unit,
            median: summary.median,
            p95: summary.p95,
            runs: RUNS,
            detail,
        }
    }
}

/// The repo root: the folder that holds the harness solution.
pub(crate) struct RepoRoot(PathBuf);

impl RepoRoot {
    fn find(start: &Path) -> Result<Self, BenchError> {
        start
            .ancestors()
            .find(|d: &&Path| d.join(HARNESS_SLNX).is_file())
            .map(|d: &Path| Self(d.to_path_buf()))
            .ok_or_else(|| BenchError::NoRepo(start.to_path_buf()))
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn harness(&self) -> PathBuf {
        self.0.join(HARNESS_SLNX)
    }

    fn fixtures(&self) -> PathBuf {
        self.0.join("fixtures")
    }
}

/// Run the set and print the table, then the JSON lines.
pub(crate) fn run(solution: Option<&Path>) -> Result<(), CommandError> {
    let cwd: PathBuf =
        std::env::current_dir().map_err(|source: std::io::Error| BenchError::Io {
            what: "read the current folder",
            source,
        })?;
    let repo: RepoRoot = RepoRoot::find(&cwd)?;
    let mut rows: Vec<Row> = vec![bb_items::cold_start()?];
    rows.extend(bb_items::outlines(&repo.fixtures())?);
    rows.push(bb_items::scan("scan-harness", &repo.harness())?);
    if let Some(sln) = solution {
        rows.push(bb_items::scan("scan-solution", sln)?);
    }
    rows.push(bb_items::audit(repo.path())?);
    rows.push(bb_items::binary_size()?);
    println!(
        "{:<16} {:>10} {:>10}  {:<5}  detail",
        "item", "median", "p95", "unit"
    );
    for row in &rows {
        println!("{}", table_line(row));
    }
    for row in &rows {
        let line: String = serde_json::to_string(row).map_err(BenchError::Json)?;
        println!("{line}");
    }
    Ok(())
}

fn table_line(row: &Row) -> String {
    let unit: &str = match row.unit {
        Unit::Micros => "us",
        Unit::Bytes => "bytes",
    };
    format!(
        "{:<16} {:>10} {:>10}  {unit:<5}  {}",
        row.item, row.median, row.p95, row.detail
    )
}
