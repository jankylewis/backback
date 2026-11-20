//! The ticks saved per workspace, kept next to the recent list: a header
//! line, then one `picked<TAB>keys<TAB>path` line per workspace, newest
//! first. Only a user's pick is saved, and each record says so (BBEOD41).
//! A v1 file mixed picks with auto-saved found sets that cannot be told
//! apart, so it reads as no choice at all (migrate: the next pick writes
//! v2). An unknown header, key, mark or line is an error, never a guess.

use std::path::{Path, PathBuf};

use crate::{Lang, LangSet};

const HEADER: &str = "backback langs v2";
/// The format before the user-picked mark (BBEOD26 to BBEOD40).
const V1_HEADER: &str = "backback langs v1";
/// The mark on every record: the user ticked these languages.
const PICKED: &str = "picked";
/// Choices kept; the oldest go first.
const KEEP: usize = 50;

/// Each workspace's ticks, newest first.
pub type Choices = Vec<(PathBuf, LangSet)>;

/// A saved choice file backback cannot read.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum StoreError {
    /// The first line is not this version's header.
    #[error("saved languages: unknown format {0:?}")]
    UnknownFormat(String),
    /// A line has no tab, an unknown language or no language.
    #[error("saved languages: bad line {0:?}")]
    BadLine(String),
}

/// The choices of a saved file; a v1 file has none.
///
/// # Errors
/// [`StoreError`] for an unknown header or a line it cannot read.
pub fn decode(text: &str) -> Result<Choices, StoreError> {
    let mut lines: std::str::Lines<'_> = text.lines();
    let header: &str = lines.next().unwrap_or_default();
    if header == V1_HEADER {
        return Ok(Vec::new());
    }
    if header != HEADER {
        return Err(StoreError::UnknownFormat(header.to_owned()));
    }
    lines
        .filter(|line: &&str| !line.trim().is_empty())
        .map(line_choice)
        .collect()
}

fn line_choice(line: &str) -> Result<(PathBuf, LangSet), StoreError> {
    let bad: &dyn Fn() -> StoreError = &|| StoreError::BadLine(line.to_owned());
    let (mark, rest): (&str, &str) = line.split_once('\t').ok_or_else(bad)?;
    if mark != PICKED {
        return Err(bad());
    }
    let (keys, path): (&str, &str) = rest.split_once('\t').ok_or_else(bad)?;
    let mut set: LangSet = LangSet::EMPTY;
    for key in keys.split(',') {
        set = set.with(Lang::from_key(key).ok_or_else(bad)?);
    }
    if set.is_empty() || path.is_empty() {
        return Err(bad());
    }
    Ok((PathBuf::from(path), set))
}

/// The text of `choices`; an empty set is never written.
#[must_use]
pub fn encode(choices: &[(PathBuf, LangSet)]) -> String {
    let mut text: String = format!("{HEADER}\n");
    for (path, set) in choices
        .iter()
        .filter(|(_, set): &&(PathBuf, LangSet)| !set.is_empty())
    {
        let keys: Vec<&str> = set.langs().map(Lang::key).collect();
        text.push_str(PICKED);
        text.push('\t');
        text.push_str(&keys.join(","));
        text.push('\t');
        text.push_str(&path.display().to_string());
        text.push('\n');
    }
    text
}

/// The ticks saved for `workspace`.
#[must_use]
pub fn choice_for(choices: &[(PathBuf, LangSet)], workspace: &Path) -> Option<LangSet> {
    choices
        .iter()
        .find(|(path, _): &&(PathBuf, LangSet)| path == workspace)
        .map(|(_, set): &(PathBuf, LangSet)| *set)
}

/// `choices` with `workspace` ticking `set`, first; an empty set changes
/// nothing (Open is disabled with no tick).
#[must_use]
pub fn with_choice(choices: Choices, workspace: &Path, set: LangSet) -> Choices {
    if set.is_empty() {
        return choices;
    }
    let mut out: Choices = choices
        .into_iter()
        .filter(|(path, _): &(PathBuf, LangSet)| path != workspace)
        .collect();
    out.insert(0, (workspace.to_path_buf(), set));
    out.truncate(KEEP);
    out
}
