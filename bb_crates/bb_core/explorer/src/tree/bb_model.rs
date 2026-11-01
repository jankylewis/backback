use std::path::{Path, PathBuf};

use crate::bb_entry::{collect_visible, find, find_mut, Entry, EntryKind};
use crate::children::{expand_entry, read_children_from};
use crate::ignore_rules;
use bb_dotnet::{Solution, SolutionError, SolutionPath};

/// Explorer I/O or input error.
#[derive(Debug, thiserror::Error)]
pub enum ExplorerError {
    /// Root is neither a folder nor a `.slnx` file.
    #[error("expected a folder or .slnx: {0}")]
    InvalidRoot(PathBuf),
    /// A path could not be read.
    #[error("cannot read {path}: {source}")]
    Read {
        /// Attempted path.
        path: PathBuf,
        /// I/O cause.
        #[source]
        source: std::io::Error,
    },
    /// Solution parsing failed.
    #[error(transparent)]
    Solution(#[from] SolutionError),
    /// No matching entry exists.
    #[error("entry not found: {0}")]
    NotFound(PathBuf),
    /// A git ignore file could not be parsed.
    #[error("cannot parse {path}: {source}")]
    Ignore {
        /// Ignore file path.
        path: PathBuf,
        /// Parse or I/O cause.
        #[source]
        source: ignore::Error,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum Source {
    Folder,
    Solution {
        directory: PathBuf,
        projects: Vec<PathBuf>,
    },
}

/// A lazy tree rooted at a folder or `.slnx`.
#[derive(Debug, Clone)]
pub struct Explorer {
    root: Entry,
    source: Source,
}

/// The root entry with the ignore rules above it and its git state.
fn root_entry(absolute: PathBuf, kind: EntryKind, source: &Source) -> Result<Entry, ExplorerError> {
    let rule_directory: &Path = match source {
        Source::Folder => &absolute,
        Source::Solution { directory, .. } => directory,
    };
    let rules: std::sync::Arc<Vec<ignore::gitignore::Gitignore>> =
        ignore_rules::ancestors(rule_directory)?;
    let in_repo: bool = ignore_rules::repo_root(rule_directory).is_some();
    Ok(Entry::new(absolute, kind, false)
        .with_rules(rules)
        .with_git(in_repo, false))
}

impl Explorer {
    /// Open a root and read its first level.
    ///
    /// # Errors
    /// Invalid input, unreadable folder, or malformed solution.
    pub fn open(path: &Path) -> Result<Self, ExplorerError> {
        let absolute: PathBuf = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|source: std::io::Error| ExplorerError::Read {
                    path: path.to_path_buf(),
                    source,
                })?
                .join(path)
        };
        let (kind, source): (EntryKind, Source) = if absolute.is_dir() {
            (EntryKind::Folder, Source::Folder)
        } else if absolute
            .extension()
            .is_some_and(|ext: &std::ffi::OsStr| ext.eq_ignore_ascii_case("slnx"))
        {
            let solution_path: SolutionPath = SolutionPath::new(absolute.clone())?;
            let solution: Solution = Solution::load(&solution_path)?;
            let directory: PathBuf = absolute.parent().unwrap_or(Path::new("/")).to_path_buf();
            let projects: Vec<PathBuf> = solution
                .projects
                .iter()
                .map(|project: &bb_dotnet::ProjectPath| project.as_path().to_path_buf())
                .collect();
            (
                EntryKind::Solution,
                Source::Solution {
                    directory,
                    projects,
                },
            )
        } else {
            return Err(ExplorerError::InvalidRoot(absolute));
        };
        let mut tree: Self = Self {
            root: root_entry(absolute, kind, &source)?,
            source,
        };
        let children: Vec<Entry> = tree.read_children(&tree.root)?;
        tree.root.children = Some(children);
        tree.root.expanded = true;
        Ok(tree)
    }

    /// Root entry.
    #[must_use]
    pub fn root(&self) -> &Entry {
        &self.root
    }

    /// Expand or collapse a folder or solution entry.
    ///
    /// # Errors
    /// The entry does not exist or its folder cannot be read.
    pub fn toggle(&mut self, path: &Path) -> Result<(), ExplorerError> {
        let entry: &Entry =
            find(&self.root, path).ok_or_else(|| ExplorerError::NotFound(path.to_path_buf()))?;
        if !entry.expandable() {
            return Ok(());
        }
        let children: Option<Vec<Entry>> = if entry.children.is_none() {
            Some(self.read_children(entry)?)
        } else {
            None
        };
        let target: &mut Entry = find_mut(&mut self.root, path)
            .ok_or_else(|| ExplorerError::NotFound(path.to_path_buf()))?;
        if let Some(children) = children {
            target.children = Some(children);
        }
        target.expanded = !target.expanded;
        Ok(())
    }

    /// Visible entries in display order, with indentation depth.
    #[must_use]
    pub fn visible(&self) -> Vec<(usize, &Entry)> {
        let mut rows: Vec<(usize, &Entry)> = Vec::new();
        collect_visible(&self.root, 0, &mut rows);
        rows
    }

    /// Expand every reachable folder, returning the count below the root.
    ///
    /// # Errors
    /// A descendant folder cannot be read.
    pub fn expand_all(&mut self) -> Result<usize, ExplorerError> {
        Ok(expand_entry(&self.source, &mut self.root)?.saturating_sub(1))
    }

    fn read_children(&self, parent: &Entry) -> Result<Vec<Entry>, ExplorerError> {
        read_children_from(&self.source, parent)
    }
}
