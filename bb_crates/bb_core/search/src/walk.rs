//! The walk: files in name order, one hit per matching line, a cap, a stop.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use ignore::overrides::Override;
use ignore::{DirEntry, WalkBuilder};

use crate::{Filter, Query};

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
/// Larger files are skipped: generated or data, not source.
const MAX_BYTES: u64 = 2 * 1024 * 1024;
/// A preview keeps at most this many chars.
const PREVIEW_CHARS: usize = 200;

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
    for path in walk(root, filter) {
        if stop.load(Ordering::Relaxed) {
            found.stopped = true;
            return found;
        }
        let Some(text): Option<String> = read_text(&path) else {
            continue;
        };
        found.files = found.files.saturating_add(1);
        if !query.may_match(&text) {
            continue;
        }
        for (index, line) in text.lines().enumerate() {
            let Some(start): Option<usize> = query.find(line) else {
                continue;
            };
            if found.hits.len() >= cap {
                found.capped = true;
                return found;
            }
            found.hits.push(hit(&path, index, line, start));
        }
    }
    found
}

/// Every file under `root` a search or Go to File may show, in name order:
/// not in [`SKIP`] folders, not git-ignored, not a [`SKIP_FILES`] name.
pub fn files(root: &Path) -> impl Iterator<Item = PathBuf> {
    walk(root, &Filter::default())
}

/// [`files`] narrowed by `filter`; a filter that cannot build keeps all.
fn walk(root: &Path, filter: &Filter) -> impl Iterator<Item = PathBuf> {
    // Applied beside .gitignore, never over it: a whitelist override would
    // bring git-ignored files back.
    let rules: Option<Override> = filter
        .overrides(root)
        .ok()
        .filter(|_: &Override| !filter.is_empty());
    let dirs: Option<Override> = rules.clone();
    WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .require_git(false)
        .sort_by_file_name(std::ffi::OsStr::cmp)
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
        })
        .build()
        .filter_map(Result::ok)
        .filter(move |entry: &DirEntry| {
            entry
                .file_type()
                .is_some_and(|t: std::fs::FileType| t.is_file())
                && !SKIP_FILES
                    .iter()
                    .any(|name: &&str| entry.file_name() == std::ffi::OsStr::new(name))
                && rules
                    .as_ref()
                    .is_none_or(|o: &Override| kept(o, entry.path()))
        })
        .map(DirEntry::into_path)
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

/// The file as UTF-8 text; `None` for big, binary or unreadable files.
fn read_text(path: &Path) -> Option<String> {
    let size: u64 = std::fs::metadata(path).ok()?.len();
    if size > MAX_BYTES {
        return None;
    }
    let bytes: Vec<u8> = std::fs::read(path).ok()?;
    if bytes.iter().take(8 * 1024).any(|b: &u8| *b == 0) {
        return None;
    }
    String::from_utf8(bytes).ok()
}

fn hit(path: &Path, index: usize, line: &str, start: usize) -> Hit {
    let before: &str = line.get(..start).unwrap_or_default();
    let units: usize = before.encode_utf16().count();
    Hit {
        path: path.to_path_buf(),
        line: u32::try_from(index).unwrap_or(u32::MAX),
        column: u32::try_from(units).unwrap_or(u32::MAX),
        preview: preview(line),
    }
}

fn preview(line: &str) -> String {
    let trimmed: &str = line.trim();
    if trimmed.chars().count() <= PREVIEW_CHARS {
        return trimmed.to_owned();
    }
    let mut short: String = trimmed.chars().take(PREVIEW_CHARS).collect();
    short.push('…');
    short
}
