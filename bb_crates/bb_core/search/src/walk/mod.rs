//! The walk: files in name order, one hit per matching line, a cap, a stop.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use ignore::overrides::Override;
use ignore::{DirEntry, WalkBuilder, WalkState};
use std::sync::{Mutex, PoisonError};

use crate::{Filter, Query};

mod bb_scan;

use bb_scan::{scan, BATCH, MAX_WORKERS};

/// One matching line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    /// The file.
    pub path: PathBuf,
    /// 0-based line.
    pub line: u32,
    /// 0-based UTF-16 column of the match start (an LSP character).
    pub column: u32,
    /// The line, trimmed and shortened for a list row.
    pub preview: String,
}

/// What a search found.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Found {
    /// Hits, file by file in name order.
    pub hits: Vec<Hit>,
    /// Files read.
    pub files: usize,
    /// True when the cap stopped the search.
    pub capped: bool,
    /// True when `stop` was set before the end.
    pub stopped: bool,
}

/// Folders never searched: build output, package caches, the VCS and IDE
/// caches. Other dot-folders (`.github`, `.config`) are searched, as VS Code
/// does (BBEOD21); `.gitignore` still rules.
const SKIP: [&str; 7] = [
    "bin",
    "obj",
    "node_modules",
    ".target",
    ".git",
    ".vs",
    ".idea",
];
/// Files never searched or listed.
const SKIP_FILES: [&str; 1] = [".DS_Store"];

/// Searches every file under `root` for `query`, up to `cap` hits. Files
/// come in name order; `stop` is read before each file.
#[must_use]
pub fn search(root: &Path, query: &Query, cap: usize, stop: &AtomicBool) -> Found {
    search_in(root, &Filter::default(), query, cap, stop)
}

/// [`search`] over only the files `filter` keeps (Find in Files' files
/// field); `root` is the folder searched (the workspace, a project or a
/// folder).
#[must_use]
pub fn search_in(
    root: &Path,
    filter: &Filter,
    query: &Query,
    cap: usize,
    stop: &AtomicBool,
) -> Found {
    let mut found: Found = Found::default();
    let workers: usize = std::thread::available_parallelism()
        .map_or(1, std::num::NonZeroUsize::get)
        .min(MAX_WORKERS);
    // First the files as the walk lists them: a search that reaches its cap
    // early never waits for the whole tree (BB-PRF-006).
    let mut head: Box<dyn Iterator<Item = PathBuf>> = Box::new(walk(root, filter).take(HEAD));
    let (mut taken, mut last): (usize, Option<PathBuf>) = (0, None);
    loop {
        let batch: Vec<PathBuf> = head.by_ref().take(BATCH).collect();
        if batch.is_empty() {
            break;
        }
        taken = taken.saturating_add(batch.len());
        last = batch.last().cloned();
        if searched(&mut found, &batch, (query, workers, cap), stop) {
            return found;
        }
    }
    if taken < HEAD {
        return found;
    }
    // Then the rest, walked on several threads; sorted, it continues the
    // same order after the last file searched.
    let rest: Vec<PathBuf> = walk_sorted(root, filter)
        .into_iter()
        .filter(|path: &PathBuf| last.as_ref().is_none_or(|end: &PathBuf| path > end))
        .collect();
    for batch in rest.chunks(BATCH) {
        if searched(&mut found, batch, (query, workers, cap), stop) {
            return found;
        }
    }
    found
}

/// Files the walk lists one by one before the rest is walked in parallel.
const HEAD: usize = 1_024;

/// Search `batch` and add its hits in file order, as one file at a time
/// would: the same hits, files count and cap. True when the search ends
/// here (stopped or capped).
fn searched(
    found: &mut Found,
    batch: &[PathBuf],
    (query, workers, cap): (&Query, usize, usize),
    stop: &AtomicBool,
) -> bool {
    let scanned: Vec<Option<Vec<Hit>>> = scan(batch, query, workers, stop);
    if stop.load(Ordering::Relaxed) {
        found.stopped = true;
        return true;
    }
    for hits in scanned.into_iter().flatten() {
        found.files = found.files.saturating_add(1);
        for one in hits {
            if found.hits.len() >= cap {
                found.capped = true;
                return true;
            }
            found.hits.push(one);
        }
    }
    false
}

/// Every file under `root` a search or Go to File may show, in name order:
/// not in [`SKIP`] folders, not git-ignored, not a [`SKIP_FILES`] name.
pub fn files(root: &Path) -> impl Iterator<Item = PathBuf> {
    walk(root, &Filter::default())
}

/// [`files`] narrowed by `filter`; a filter that cannot build keeps all.
fn walk(root: &Path, filter: &Filter) -> impl Iterator<Item = PathBuf> {
    let rules: Option<Override> = rules_of(root, filter);
    let mut walker: WalkBuilder = builder(root, rules.clone());
    let _sorted: &mut WalkBuilder = walker.sort_by_file_name(std::ffi::OsStr::cmp);
    walker
        .build()
        .filter_map(Result::ok)
        .filter(move |entry: &DirEntry| keep(entry, rules.as_ref()))
        .map(DirEntry::into_path)
}

/// [`walk`]'s files, walked on several threads and then sorted: a path
/// sorts folder by folder, by name, so the order is [`walk`]'s
/// (BB-PRF-006).
fn walk_sorted(root: &Path, filter: &Filter) -> Vec<PathBuf> {
    let rules: Option<Override> = rules_of(root, filter);
    let found: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());
    let mut walker: WalkBuilder = builder(root, rules.clone());
    let _threads: &mut WalkBuilder = walker.threads(MAX_WORKERS);
    walker.build_parallel().run(|| {
        let rules: Option<Override> = rules.clone();
        let found: &Mutex<Vec<PathBuf>> = &found;
        Box::new(move |entry: Result<DirEntry, ignore::Error>| {
            if let Ok(entry) = entry {
                if keep(&entry, rules.as_ref()) {
                    found
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .push(entry.into_path());
                }
            }
            WalkState::Continue
        })
    });
    let mut paths: Vec<PathBuf> = found.into_inner().unwrap_or_else(PoisonError::into_inner);
    paths.sort();
    paths
}

/// The files field as overrides, applied beside .gitignore, never over it:
/// a whitelist override would bring git-ignored files back.
fn rules_of(root: &Path, filter: &Filter) -> Option<Override> {
    filter
        .overrides(root)
        .ok()
        .filter(|_: &Override| !filter.is_empty())
}

/// The walk's settings: dot-folders searched, .gitignore applied, [`SKIP`]
/// and folders the files field excludes left out.
fn builder(root: &Path, dirs: Option<Override>) -> WalkBuilder {
    let mut walker: WalkBuilder = WalkBuilder::new(root);
    let _set: &mut WalkBuilder = walker
        .hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .require_git(false)
        .filter_entry(move |entry: &DirEntry| {
            let dir: bool = entry
                .file_type()
                .is_some_and(|t: std::fs::FileType| t.is_dir());
            let skipped: bool = SKIP
                .iter()
                .any(|name: &&str| entry.file_name() == std::ffi::OsStr::new(name));
            let excluded: bool = dirs
                .as_ref()
                .is_some_and(|o: &Override| o.matched(entry.path(), true).is_ignore());
            !(dir && (skipped || excluded))
        });
    walker
}

/// A file the walk lists: not a [`SKIP_FILES`] name, kept by the files field.
fn keep(entry: &DirEntry, rules: Option<&Override>) -> bool {
    entry
        .file_type()
        .is_some_and(|t: std::fs::FileType| t.is_file())
        && !SKIP_FILES
            .iter()
            .any(|name: &&str| entry.file_name() == std::ffi::OsStr::new(name))
        && rules.is_none_or(|o: &Override| kept(o, entry.path()))
}

/// A file the files field keeps: matched by an include glob, or not
/// excluded when there is no include glob.
fn kept(rules: &Override, path: &Path) -> bool {
    match rules.matched(path, false) {
        ignore::Match::Whitelist(_) => true,
        ignore::Match::Ignore(_) => false,
        ignore::Match::None => rules.num_whitelists() == 0,
    }
}
