//! Which languages a folder holds: a walk that skips dot folders and
//! git-ignored paths, stops once all three are found, and ends at a deadline
//! or an entry cap, so a huge folder never holds the window up.

use std::path::Path;
use std::time::{Duration, Instant};

use ignore::{DirEntry, WalkBuilder};

use crate::{Lang, LangSet};

/// How long [`detect`] may walk when a window opens.
pub const DETECT_BUDGET: Duration = Duration::from_millis(400);
/// At most this many entries are read, whatever the deadline.
const MAX_ENTRIES: usize = 200_000;
/// Build output that is not always git-ignored.
const SKIP: [&str; 5] = ["bin", "obj", "target", "node_modules", "__pycache__"];

/// What [`detect`] found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Detection {
    /// The languages seen.
    pub found: LangSet,
    /// False when the deadline or the entry cap ended the walk first.
    pub complete: bool,
}

/// The languages `folder` holds, read for at most `budget`.
#[must_use]
pub fn detect(folder: &Path, budget: Duration) -> Detection {
    detect_until(folder, LangSet::of(&Lang::ALL), budget)
}

/// Like [`detect`], but the walk ends once every `wanted` language is
/// seen: a C#-only window stops at its first C# file.
#[must_use]
pub fn detect_until(folder: &Path, wanted: LangSet, budget: Duration) -> Detection {
    let deadline: Instant = Instant::now()
        .checked_add(budget)
        .unwrap_or_else(Instant::now);
    let mut found: LangSet = LangSet::EMPTY;
    let mut seen: usize = 0;
    for entry in walker(folder).build().filter_map(Result::ok) {
        seen = seen.saturating_add(1);
        if let Some(lang) = Lang::of_marker(entry.path()).filter(|_: &Lang| is_file(&entry)) {
            found = found.with(lang);
        }
        if wanted.but_not(found).is_empty() {
            return Detection {
                found,
                complete: true,
            };
        }
        if seen >= MAX_ENTRIES || Instant::now() >= deadline {
            return Detection {
                found,
                complete: false,
            };
        }
    }
    Detection {
        found,
        complete: true,
    }
}

fn is_file(entry: &DirEntry) -> bool {
    entry
        .file_type()
        .is_some_and(|kind: std::fs::FileType| kind.is_file())
}

fn walker(folder: &Path) -> WalkBuilder {
    let mut walker: WalkBuilder = WalkBuilder::new(folder);
    let _set: &mut WalkBuilder = walker
        .hidden(true)
        .git_ignore(true)
        .git_exclude(true)
        .require_git(false)
        .follow_links(false)
        .filter_entry(|entry: &DirEntry| {
            let dir: bool = entry
                .file_type()
                .is_some_and(|kind: std::fs::FileType| kind.is_dir());
            !(dir && SKIP.iter().any(|name: &&str| entry.file_name() == *name))
        });
    walker
}
