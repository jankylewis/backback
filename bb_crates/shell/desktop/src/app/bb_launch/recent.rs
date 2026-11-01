//! The workspaces opened lately, newest first (BBEOD13). A small versioned
//! text file in `$BACKBACK_STATE_DIR`, else `~/Library/Application
//! Support/backback`: a header line, then one path per line.

use std::path::{Path, PathBuf};

const HEADER: &str = "backback recent v1";
const KEEP: usize = 8;
const FILE: &str = "recent";

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

/// `path` first, once, and at most `KEEP` paths.
pub(crate) fn add(paths: Vec<PathBuf>, path: PathBuf) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = paths.into_iter().filter(|p: &PathBuf| *p != path).collect();
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

fn read(dir: &Path) -> Vec<PathBuf> {
    match std::fs::read_to_string(dir.join(FILE)) {
        Ok(text) => decode(&text).unwrap_or_else(|error: RecentError| {
            eprintln!("backback: {error}");
            Vec::new()
        }),
        Err(_) => Vec::new(),
    }
}

/// The newest recent workspace that still exists.
pub(crate) fn last() -> Option<PathBuf> {
    read(&state_dir()?)
        .into_iter()
        .find(|path: &PathBuf| path.exists())
}

/// Put `path` first in the recent list; a write error is reported, not fatal.
pub(crate) fn remember(path: &Path) {
    let Some(dir): Option<PathBuf> = state_dir() else {
        return;
    };
    let paths: Vec<PathBuf> = add(read(&dir), path.to_path_buf());
    let tmp: PathBuf = dir.join(format!("{FILE}.tmp"));
    let written: std::io::Result<()> = std::fs::create_dir_all(&dir)
        .and_then(|(): ()| std::fs::write(&tmp, encode(&paths)))
        .and_then(|(): ()| std::fs::rename(&tmp, dir.join(FILE)));
    if let Err(error) = written {
        eprintln!("backback: cannot save recent workspaces: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_goes_first_once_and_the_list_keeps_eight() {
        let old: Vec<PathBuf> = (0..9)
            .map(|n: i32| PathBuf::from(format!("/w{n}")))
            .collect();
        let list: Vec<PathBuf> = add(old, PathBuf::from("/w3"));
        assert_eq!(list.first(), Some(&PathBuf::from("/w3")));
        assert_eq!(list.len(), 8);
        assert_eq!(
            list.iter()
                .filter(|p: &&PathBuf| p.as_path() == Path::new("/w3"))
                .count(),
            1
        );
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
