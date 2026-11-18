//! Which projects of the solution Roslyn has loaded, by distinct path.
//!
//! Roslyn writes one line per project (`Successfully completed load of
//! <path>` or `Error while loading <path>: <reason>`) and one summary line
//! per batch (`Completed (re)load of N project(s)`). A batch can reload a
//! project it already loaded (bus 066: `_Core.csproj` twice), so a sum of
//! batch counts overshoots. Only the set of distinct paths says what is
//! loaded; the batch lines are never counted.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use bb_dotnet::ProjectPath;

/// The line Roslyn writes after one project loaded.
const LOADED_MARK: &str = "Successfully completed load of ";
/// The line Roslyn writes when one project failed to load.
const FAILED_MARK: &str = "Error while loading ";

/// One spelling-independent identity of a project file: canonical when the
/// file exists, else the absolute path with `.` and `..` folded.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ProjectKey(PathBuf);

impl ProjectKey {
    fn of(path: &Path) -> Self {
        Self(std::fs::canonicalize(path).unwrap_or_else(|_: std::io::Error| fold(path)))
    }
}

fn fold(path: &Path) -> PathBuf {
    let mut out: PathBuf = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => out.push(part),
        }
    }
    out
}

/// What one log line says about one project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Loaded,
    Failed,
}

fn outcome(text: &str) -> Option<(Outcome, &str)> {
    if let Some((_, rest)) = text.split_once(LOADED_MARK) {
        return Some((Outcome::Loaded, rest.trim_end()));
    }
    let (_, rest): (&str, &str) = text.split_once(FAILED_MARK)?;
    let path: &str = rest.split_once(": ").map_or(rest, |(p, _): (&str, &str)| p);
    Some((Outcome::Failed, path.trim_end()))
}

/// The load state of every project the solution declares.
#[derive(Debug, Clone)]
pub struct LoadState {
    expected: BTreeMap<ProjectKey, ProjectPath>,
    loaded: BTreeSet<ProjectKey>,
    failed: BTreeSet<ProjectKey>,
}

impl LoadState {
    /// Track `projects`; duplicates in the listing count once.
    #[must_use]
    pub fn new(projects: &[ProjectPath]) -> Self {
        let expected: BTreeMap<ProjectKey, ProjectPath> = projects
            .iter()
            .map(|p: &ProjectPath| (ProjectKey::of(p.as_path()), p.clone()))
            .collect();
        Self {
            expected,
            loaded: BTreeSet::new(),
            failed: BTreeSet::new(),
        }
    }

    /// Read one log line. Returns `true` when it changed the state. A line
    /// for a project outside the solution, or a repeat, changes nothing.
    pub fn observe(&mut self, text: &str) -> bool {
        let Some((what, path)): Option<(Outcome, &str)> = outcome(text) else {
            return false;
        };
        let key: ProjectKey = ProjectKey::of(Path::new(path));
        if !self.expected.contains_key(&key) {
            return false;
        }
        match what {
            Outcome::Loaded => {
                let retried: bool = self.failed.remove(&key);
                self.loaded.insert(key) || retried
            }
            Outcome::Failed if self.loaded.contains(&key) => false,
            Outcome::Failed => self.failed.insert(key),
        }
    }

    /// Distinct projects of the solution loaded so far.
    #[must_use]
    pub fn loaded(&self) -> usize {
        self.loaded.len()
    }

    /// Distinct projects the solution declares.
    #[must_use]
    pub fn total(&self) -> usize {
        self.expected.len()
    }

    /// Every project is loaded or has failed: waiting longer gains nothing.
    #[must_use]
    pub fn settled(&self) -> bool {
        self.loaded.len().saturating_add(self.failed.len()) >= self.expected.len()
    }

    /// Projects that failed to load and never loaded after.
    #[must_use]
    pub fn failed(&self) -> Vec<ProjectPath> {
        self.pick(|key: &ProjectKey| self.failed.contains(key))
    }

    /// Projects with no line yet: neither loaded nor failed.
    #[must_use]
    pub fn missing(&self) -> Vec<ProjectPath> {
        self.pick(|key: &ProjectKey| !self.loaded.contains(key) && !self.failed.contains(key))
    }

    fn pick(&self, keep: impl Fn(&ProjectKey) -> bool) -> Vec<ProjectPath> {
        self.expected
            .iter()
            .filter(|(key, _): &(&ProjectKey, &ProjectPath)| keep(key))
            .map(|(_, path): (&ProjectKey, &ProjectPath)| path.clone())
            .collect()
    }
}
