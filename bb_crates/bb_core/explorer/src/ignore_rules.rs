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
    let mut rules: Arc<Vec<Gitignore>> = Arc::new(Vec::new());
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
