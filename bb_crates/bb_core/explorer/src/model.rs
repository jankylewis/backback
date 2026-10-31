use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::entry::{
    collect_visible, dim_name, file_kind, find, find_mut, sort_entries, Entry, EntryKind,
};
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
}

#[derive(Debug, Clone)]
enum Source {
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
            root: Entry::new(absolute, kind, false),
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

fn read_children_from(source: &Source, parent: &Entry) -> Result<Vec<Entry>, ExplorerError> {
    match source {
        Source::Folder => read_folder(&parent.path, parent.dimmed),
        Source::Solution {
            directory,
            projects,
        } => {
            let location: &Path = if parent.kind == EntryKind::Solution {
                directory
            } else {
                &parent.path
            };
            Ok(solution_children(location, projects, parent.dimmed))
        }
    }
}

fn expand_entry(source: &Source, entry: &mut Entry) -> Result<usize, ExplorerError> {
    if !entry.expandable() {
        return Ok(0);
    }
    if entry.children.is_none() {
        entry.children = Some(read_children_from(source, entry)?);
    }
    entry.expanded = true;
    let mut count: usize = 1;
    if let Some(children) = &mut entry.children {
        for child in children {
            count = count.saturating_add(expand_entry(source, child)?);
        }
    }
    Ok(count)
}

fn read_folder(path: &Path, inherited_dim: bool) -> Result<Vec<Entry>, ExplorerError> {
    let items: fs::ReadDir =
        fs::read_dir(path).map_err(|source: std::io::Error| ExplorerError::Read {
            path: path.to_path_buf(),
            source,
        })?;
    let mut children: Vec<Entry> = Vec::new();
    for item in items {
        let item: fs::DirEntry = item.map_err(|source: std::io::Error| ExplorerError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let metadata: fs::Metadata =
            fs::symlink_metadata(item.path()).map_err(|source: std::io::Error| {
                ExplorerError::Read {
                    path: item.path(),
                    source,
                }
            })?;
        let kind: EntryKind = if metadata.is_dir() {
            EntryKind::Folder
        } else if metadata.is_file() {
            file_kind(&item.path())
        } else {
            EntryKind::Other
        };
        let dimmed: bool = inherited_dim || dim_name(&item.file_name().to_string_lossy());
        children.push(Entry::new(item.path(), kind, dimmed));
    }
    sort_entries(&mut children);
    Ok(children)
}

fn solution_children(parent: &Path, projects: &[PathBuf], inherited_dim: bool) -> Vec<Entry> {
    let mut children: Vec<Entry> = Vec::new();
    for project in projects {
        let Ok(relative): Result<&Path, _> = project.strip_prefix(parent) else {
            continue;
        };
        let Some(component): Option<&std::ffi::OsStr> = relative
            .components()
            .next()
            .map(std::path::Component::as_os_str)
        else {
            continue;
        };
        let path: PathBuf = parent.join(component);
        if children.iter().any(|entry: &Entry| entry.path == path) {
            continue;
        }
        let kind: EntryKind = if &path == project {
            EntryKind::Project
        } else {
            EntryKind::Folder
        };
        let dimmed: bool = inherited_dim || dim_name(&component.to_string_lossy());
        children.push(Entry::new(path, kind, dimmed));
    }
    sort_entries(&mut children);
    children
}
