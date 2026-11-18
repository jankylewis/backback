//! The workspaces opened lately, newest first (BBEOD13). A small versioned
//! text file in `$BACKBACK_STATE_DIR`, else `~/Library/Application
//! Support/backback`: a header line, then one path per line. The language
//! ticks of each workspace sit next to it, in `langs` (BBEOD26).

use std::path::{Path, PathBuf};

const HEADER: &str = "backback recent v1";
/// As many as VS Code's File > Open Recent shows first.
const KEEP: usize = 10;
const FILE: &str = "recent";
/// The ticks per workspace, next to the recent list (BBEOD26).
const LANGS: &str = "langs";

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum RecentError {
    #[error("recent workspaces: unknown format {0:?}")]
    UnknownFormat(String),
}

/// The paths of a recent file; an unknown header is an error, never a guess.
pub(crate) fn decode(text: &str) -> Result<Vec<PathBuf>, RecentError> {
    let mut lines: std::str::Lines<'_> = text.lines();
    let header: &str = lines.next().unwrap_or_default();
    if header != HEADER {
        return Err(RecentError::UnknownFormat(header.to_owned()));
    }
    Ok(lines
        .filter(|line: &&str| !line.trim().is_empty())
        .map(PathBuf::from)
        .collect())
}

pub(crate) fn encode(paths: &[PathBuf]) -> String {
    let mut text: String = format!("{HEADER}\n");
    for path in paths {
        text.push_str(&path.display().to_string());
        text.push('\n');
    }
    text
}

/// `path` first, once, then the others still on disk (a deleted folder,
/// such as a test run's temp folder, gives its place up), at most `KEEP`.
pub(crate) fn add(paths: Vec<PathBuf>, path: PathBuf) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = paths
        .into_iter()
        .filter(|p: &PathBuf| *p != path && p.exists())
        .collect();
    out.insert(0, path);
    out.truncate(KEEP);
    out
}

/// Where backback keeps its small state files.
pub(crate) fn state_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("BACKBACK_STATE_DIR") {
        return Some(PathBuf::from(dir));
    }
    let home: PathBuf = PathBuf::from(std::env::var_os("HOME")?);
    Some(home.join("Library/Application Support/backback"))
}

/// The state folder of the recent list and the language ticks; unit tests
/// never read or write the real one (the settings seam): only a folder a
/// test sets with [`test_store`].
fn store() -> Option<PathBuf> {
    if cfg!(test) {
        return TEST_STORE.with(|dir: &std::cell::RefCell<Option<PathBuf>>| dir.borrow().clone());
    }
    state_dir()
}

thread_local! {
    static TEST_STORE: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

/// Window tests: keep the state files in `dir` on this thread (None: none).
#[cfg(test)]
pub(crate) fn test_store(dir: Option<PathBuf>) {
    TEST_STORE.with(|cell: &std::cell::RefCell<Option<PathBuf>>| cell.replace(dir));
}

/// The language ticks saved for `workspace` (BBEOD26).
pub(crate) fn langs_of(workspace: &Path) -> Option<bb_lang::LangSet> {
    let dir: PathBuf = store()?;
    bb_lang::choice_for(&read_langs(&dir), workspace)
}

/// Save `ticks` for `workspace`; an empty set is never saved.
pub(crate) fn save_langs(workspace: &Path, ticks: bb_lang::LangSet) {
    let Some(dir): Option<PathBuf> = store() else {
        return;
    };
    let choices: bb_lang::Choices = bb_lang::with_choice(read_langs(&dir), workspace, ticks);
    if let Err(error) = write_atomic(&dir, LANGS, &bb_lang::encode(&choices)) {
        eprintln!("backback: cannot save the languages: {error}");
    }
}

fn read_langs(dir: &Path) -> bb_lang::Choices {
    match std::fs::read_to_string(dir.join(LANGS)) {
        Ok(text) => bb_lang::decode(&text).unwrap_or_else(|error: bb_lang::StoreError| {
            eprintln!("backback: {error}");
            Vec::new()
        }),
        Err(_) => Vec::new(),
    }
}

fn write_atomic(dir: &Path, name: &str, text: &str) -> std::io::Result<()> {
    let tmp: PathBuf = dir.join(format!("{name}.tmp"));
    std::fs::create_dir_all(dir)
        .and_then(|(): ()| std::fs::write(&tmp, text))
        .and_then(|(): ()| std::fs::rename(&tmp, dir.join(name)))
}

fn read(dir: &Path) -> Vec<PathBuf> {
    match std::fs::read_to_string(dir.join(FILE)) {
        Ok(text) => decode(&text).unwrap_or_else(|error: RecentError| {
            eprintln!("backback: {error}");
            Vec::new()
        }),
        Err(_) => Vec::new(),
    }
}

/// Every recent workspace, newest first, as saved (the start page leaves out
/// the ones gone from disk).
pub(crate) fn list() -> Vec<PathBuf> {
    store().map_or_else(Vec::new, |dir: PathBuf| read(&dir))
}

/// Put `path` first in the recent list; a write error is reported, not fatal.
pub(crate) fn remember(path: &Path) {
    let Some(dir): Option<PathBuf> = store() else {
        return;
    };
    let paths: Vec<PathBuf> = add(read(&dir), path.to_path_buf());
    if let Err(error) = write_atomic(&dir, FILE, &encode(&paths)) {
        eprintln!("backback: cannot save recent workspaces: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_goes_first_once_and_the_list_keeps_ten_still_on_disk() {
        let base: tempfile::TempDir = tempfile::tempdir().unwrap();
        let old: Vec<PathBuf> = (0..12)
            .map(|n: i32| {
                let path: PathBuf = base.path().join(format!("w{n}"));
                std::fs::create_dir(&path).unwrap();
                path
            })
            .chain([base.path().join("gone")])
            .collect();
        let w3: PathBuf = base.path().join("w3");
        let list: Vec<PathBuf> = add(old.clone(), w3.clone());
        assert_eq!(list.first(), Some(&w3));
        assert_eq!(list.len(), 10);
        assert_eq!(list.iter().filter(|p: &&PathBuf| **p == w3).count(), 1);
        let gone: PathBuf = base.path().join("gone");
        let short: Vec<PathBuf> = add(vec![gone.clone(), w3.clone()], base.path().join("w1"));
        assert_eq!(short.len(), 2, "a deleted folder is dropped: {short:?}");
        assert!(!short.contains(&gone));
    }

    #[test]
    fn encode_then_decode_gives_the_same_paths_and_an_unknown_header_is_refused() {
        let paths: Vec<PathBuf> = vec![PathBuf::from("/a b/Sln"), PathBuf::from("/c")];
        assert_eq!(decode(&encode(&paths)), Ok(paths));
        assert_eq!(
            decode("backback recent v9\n/a\n"),
            Err(RecentError::UnknownFormat("backback recent v9".to_owned()))
        );
    }
}
