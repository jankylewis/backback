use std::{path::Path, sync::Arc};

use ignore::{
    gitignore::{Gitignore, GitignoreBuilder},
    Match,
};

use crate::model::ExplorerError;

pub(super) fn extend(
    inherited: &Arc<Vec<Gitignore>>,
    directory: &Path,
) -> Result<Arc<Vec<Gitignore>>, ExplorerError> {
    let file: std::path::PathBuf = directory.join(".gitignore");
    if !file.is_file() {
        return Ok(Arc::clone(inherited));
    }
    let mut builder: GitignoreBuilder = GitignoreBuilder::new(directory);
    if let Some(source) = builder.add(&file) {
        return Err(ExplorerError::Ignore { path: file, source });
    }
    let matcher: Gitignore = builder
        .build()
        .map_err(|source: ignore::Error| ExplorerError::Ignore { path: file, source })?;
    let mut rules: Vec<Gitignore> = inherited.as_ref().clone();
    rules.push(matcher);
    Ok(Arc::new(rules))
}

/// The git work tree containing `directory`, if any.
pub(super) fn repo_root(directory: &Path) -> Option<std::path::PathBuf> {
    directory
        .ancestors()
        .find(|dir: &&Path| dir.join(".git").exists())
        .map(Path::to_path_buf)
}

/// Git's repo-wide rules, lowest precedence first: the global excludes file,
/// then `.git/info/exclude`.
fn repo_wide(root: &Path) -> Result<Vec<Gitignore>, ExplorerError> {
    let (global, _unreadable): (Gitignore, Option<ignore::Error>) = Gitignore::global();
    let mut rules: Vec<Gitignore> = vec![global];
    let exclude: std::path::PathBuf = root.join(".git/info/exclude");
    if exclude.is_file() {
        let mut builder: GitignoreBuilder = GitignoreBuilder::new(root);
        if let Some(source) = builder.add(&exclude) {
            return Err(ExplorerError::Ignore {
                path: exclude,
                source,
            });
        }
        rules.push(
            builder
                .build()
                .map_err(|source: ignore::Error| ExplorerError::Ignore {
                    path: exclude,
                    source,
                })?,
        );
    }
    Ok(rules)
}

pub(super) fn ancestors(directory: &Path) -> Result<Arc<Vec<Gitignore>>, ExplorerError> {
    let mut directories: Vec<std::path::PathBuf> = Vec::new();
    let mut current: Option<&Path> = directory.parent();
    while let Some(parent) = current {
        directories.push(parent.to_path_buf());
        if parent.join(".git").exists() {
            break;
        }
        current = parent.parent();
    }
    directories.reverse();
    let base: Vec<Gitignore> = match repo_root(directory) {
        Some(root) => repo_wide(&root)?,
        None => Vec::new(),
    };
    let mut rules: Arc<Vec<Gitignore>> = Arc::new(base);
    for parent in directories {
        rules = extend(&rules, &parent)?;
    }
    Ok(rules)
}

pub(super) fn ignored(rules: &[Gitignore], path: &Path, is_dir: bool) -> bool {
    for matcher in rules.iter().rev() {
        match matcher.matched(path, is_dir) {
            Match::Ignore(_) => return true,
            Match::Whitelist(_) => return false,
            Match::None => {}
        }
    }
    false
}

pub(super) fn hidden(name: &str) -> bool {
    matches!(name, ".DS_Store" | ".git")
}
