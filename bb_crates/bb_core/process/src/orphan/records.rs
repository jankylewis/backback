//! What a backback process started, on disk, so the next launch can end
//! what an earlier one left behind (BB-PRC-009): the guard is the first
//! line, this is the second, for when the guard itself died with its owner.
//!
//! One file per process, `<dir>/<pid>`, rewritten on every change:
//!
//! ```text
//! backback children v1
//! owner <pid> <start>
//! group <pgid> <start>
//! ```
//!
//! `<start>` is the `lstart` token ([`super::started`]). A file whose owner
//! is the same live process (pid and token) belongs to a running backback
//! and is left alone. Otherwise each group is ended only when its leader is
//! the process the file names or is gone while members remain, then the file
//! goes. A file this reader does not know is refused: kept, nothing ended.

use std::fmt::Write;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use crate::bb_group::GroupId;

const HEADER: &str = "backback children v1";

/// One record: its owner and the groups it watched.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Record {
    pub(crate) owner: (u32, String),
    pub(crate) groups: Vec<(u32, String)>,
}

/// What a sweep did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Swept {
    /// The groups it ended.
    pub ended: Vec<u32>,
    /// Files it refused (unknown format), kept as they are.
    pub refused: Vec<PathBuf>,
}

/// The file's text.
pub(crate) fn encode(record: &Record) -> String {
    let mut text: String = format!("{HEADER}\nowner {} {}\n", record.owner.0, record.owner.1);
    for (group, started) in &record.groups {
        let _written: std::fmt::Result = writeln!(text, "group {group} {started}");
    }
    text
}

/// A record, or `None` for a format this reader does not know.
pub(crate) fn decode(text: &str) -> Option<Record> {
    let mut lines: std::str::Lines<'_> = text.lines();
    if lines.next()? != HEADER {
        return None;
    }
    let owner: (u32, String) = entry(lines.next()?, "owner")?;
    let groups: Option<Vec<(u32, String)>> = lines
        .filter(|line: &&str| !line.trim().is_empty())
        .map(|line: &str| entry(line, "group"))
        .collect();
    Some(Record {
        owner,
        groups: groups?,
    })
}

fn entry(line: &str, kind: &str) -> Option<(u32, String)> {
    let mut words: std::str::SplitWhitespace<'_> = line.split_whitespace();
    if words.next()? != kind {
        return None;
    }
    let pid: u32 = words.next()?.parse().ok()?;
    let started: String = words.next()?.to_owned();
    words.next().is_none().then_some((pid, started))
}

/// Write `record` as `<dir>/<owner pid>`; best effort (a full disk only
/// loses the backstop, never the guard).
pub(crate) fn write(dir: &Path, record: &Record) {
    let file: PathBuf = dir.join(record.owner.0.to_string());
    let _written: std::io::Result<()> = std::fs::write(file, encode(record));
}

/// Remove this owner's file (a clean quit).
pub(crate) fn remove(dir: &Path, owner: u32) {
    let _removed: std::io::Result<()> = std::fs::remove_file(dir.join(owner.to_string()));
}

/// End what dead owners in `dir` left; see the module notes.
#[must_use]
pub fn sweep(dir: &Path) -> Swept {
    let mut swept: Swept = Swept::default();
    let Ok(entries): std::io::Result<std::fs::ReadDir> = std::fs::read_dir(dir) else {
        return swept;
    };
    for entry in entries.flatten() {
        let path: PathBuf = entry.path();
        let named: bool = entry.file_name().to_str().is_some_and(|name: &str| {
            !name.is_empty() && name.bytes().all(|b: u8| b.is_ascii_digit())
        });
        if !named {
            continue;
        }
        let Some(record): Option<Record> = std::fs::read_to_string(&path)
            .ok()
            .and_then(|text: String| decode(&text))
        else {
            swept.refused.push(path);
            continue;
        };
        if is_running(&record.owner) {
            continue;
        }
        for (group, started) in &record.groups {
            if let Some(id) = ours(*group, started) {
                id.end_now();
                swept.ended.push(*group);
            }
        }
        let _removed: std::io::Result<()> = std::fs::remove_file(&path);
    }
    swept
}

/// The owner is the same live process the file names.
fn is_running((pid, started): &(u32, String)) -> bool {
    super::started(*pid).is_some_and(|now: String| &now == started)
}

/// `group` when it is still the one the record saw start: its leader is
/// the same process, or gone while members remain.
fn ours(group: u32, started: &str) -> Option<GroupId> {
    let id: GroupId = GroupId::foreign(NonZeroU32::new(group)?);
    match super::started(group) {
        Some(now) => (now == started).then_some(id),
        None => id.alive().then_some(id),
    }
}
