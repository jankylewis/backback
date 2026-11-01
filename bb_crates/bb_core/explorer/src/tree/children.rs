use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::bb_entry::{dim_name, file_kind, sort_entries, Entry, EntryKind};
use crate::ignore_rules;
use crate::model::{ExplorerError, Source};

pub(super) fn read_children_from(
    source: &Source,
    parent: &Entry,
) -> Result<Vec<Entry>, ExplorerError> {
    match source {
        Source::Folder => read_folder(parent),
        Source::Solution {
            directory,
            projects,
        } => {
            let location: &Path = if parent.kind == EntryKind::Solution {
                directory
            } else {
                &parent.path
            };
            let rules: std::sync::Arc<Vec<ignore::gitignore::Gitignore>> =
                ignore_rules::extend(&parent.ignore_rules, location)?;
            Ok(solution_children(location, projects, parent, &rules))
        }
    }
}

pub(super) fn expand_entry(source: &Source, entry: &mut Entry) -> Result<usize, ExplorerError> {
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

/// The parent's rules, plus the folder's own `.gitignore` when it has one.
fn folder_rules(
    parent: &Entry,
    items: &[fs::DirEntry],
) -> Result<std::sync::Arc<Vec<ignore::gitignore::Gitignore>>, ExplorerError> {
    if items
        .iter()
        .any(|item: &fs::DirEntry| item.file_name() == ".gitignore")
    {
        ignore_rules::extend(&parent.ignore_rules, &parent.path)
    } else {
        Ok(parent.ignore_rules.clone())
    }
}

fn read_folder(parent: &Entry) -> Result<Vec<Entry>, ExplorerError> {
    let path: &Path = &parent.path;
    let items: Vec<fs::DirEntry> = fs::read_dir(path)
        .map_err(|source: std::io::Error| ExplorerError::Read {
            path: path.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<fs::DirEntry>, std::io::Error>>()
        .map_err(|source: std::io::Error| ExplorerError::Read {
            path: path.to_path_buf(),
            source,
        })?;
    let rules: std::sync::Arc<Vec<ignore::gitignore::Gitignore>> = folder_rules(parent, &items)?;
    let mut children: Vec<Entry> = Vec::new();
    for item in items {
        let name: String = item.file_name().to_string_lossy().into_owned();
        if ignore_rules::hidden(&name) {
            continue;
        }
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
        let ignored: bool =
            parent.git.ignored || ignore_rules::ignored(&rules, &item.path(), metadata.is_dir());
        let dimmed: bool = parent.dimmed || dim_name(&name) || ignored;
        children.push(
            Entry::new(item.path(), kind, dimmed)
                .with_rules(rules.clone())
                .with_git(parent.git.in_repo, ignored),
        );
    }
    sort_entries(&mut children);
    Ok(children)
}

fn solution_children(
    parent: &Path,
    projects: &[PathBuf],
    owner: &Entry,
    rules: &std::sync::Arc<Vec<ignore::gitignore::Gitignore>>,
) -> Vec<Entry> {
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
        if ignore_rules::hidden(&component.to_string_lossy()) {
            continue;
        }
        if children.iter().any(|entry: &Entry| entry.path == path) {
            continue;
        }
        let kind: EntryKind = if &path == project {
            EntryKind::Project
        } else {
            EntryKind::Folder
        };
        let ignored: bool =
            owner.git.ignored || ignore_rules::ignored(rules, &path, kind == EntryKind::Folder);
        let dimmed: bool = owner.dimmed || dim_name(&component.to_string_lossy()) || ignored;
        children.push(
            Entry::new(path, kind, dimmed)
                .with_rules(rules.clone())
                .with_git(owner.git.in_repo, ignored),
        );
    }
    sort_entries(&mut children);
    children
}
