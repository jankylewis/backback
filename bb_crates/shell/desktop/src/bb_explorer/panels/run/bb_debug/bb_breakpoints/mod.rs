//! Breakpoints (BBEOD17): 0-based lines per file, toggled with F9 or a click
//! on the line number. They stay on their code when lines are added or
//! removed above them, as in VS Code and Rider.

mod bb_exceptions;
mod toggle;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(crate) use bb_exceptions::{ExceptionBreaks, ExceptionKind};
pub(crate) use toggle::{after_edit, for_start, on_toggle, toggle_line, ToggleBreakpoint};

/// What a new session sends, and the deleted files whose breakpoints were
/// dropped.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ForSession {
    pub(crate) sent: Vec<(PathBuf, Vec<usize>)>,
    pub(crate) gone: Vec<PathBuf>,
}

impl ForSession {
    /// The status line naming the dropped files, if any.
    pub(crate) fn note(&self) -> Option<String> {
        let names: Vec<String> = self
            .gone
            .iter()
            .map(|path: &PathBuf| {
                path.file_name().map_or_else(
                    || path.display().to_string(),
                    |name: &std::ffi::OsStr| name.to_string_lossy().into_owned(),
                )
            })
            .collect();
        (!names.is_empty()).then(|| {
            format!(
                "Debug: breakpoints in {} dropped: the file no longer exists",
                names.join(", ")
            )
        })
    }
}

/// Every breakpoint, by file.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Breakpoints {
    by_file: BTreeMap<PathBuf, BTreeSet<usize>>,
}

impl Breakpoints {
    /// Set or clear the breakpoint on `line`; true when it is now set.
    pub(crate) fn toggle(&mut self, path: &Path, line: usize) -> bool {
        let lines: &mut BTreeSet<usize> = self.by_file.entry(path.to_path_buf()).or_default();
        let set: bool = if lines.remove(&line) {
            false
        } else {
            lines.insert(line)
        };
        if lines.is_empty() {
            self.by_file.remove(path);
        }
        set
    }

    /// True when `line` of `path` has a breakpoint.
    pub(crate) fn has(&self, path: &Path, line: usize) -> bool {
        self.by_file
            .get(path)
            .is_some_and(|lines: &BTreeSet<usize>| lines.contains(&line))
    }

    /// The breakpoint lines of `path`, ascending.
    pub(crate) fn lines(&self, path: &Path) -> Vec<usize> {
        self.by_file
            .get(path)
            .map(|lines: &BTreeSet<usize>| lines.iter().copied().collect())
            .unwrap_or_default()
    }

    /// The breakpoints a new session sends. A file no longer on disk cannot
    /// bind (BB-BUG-082): its breakpoints are dropped here and the file is
    /// returned in `gone`, so the start can say so.
    pub(crate) fn for_session(&mut self) -> ForSession {
        let gone: Vec<PathBuf> = self
            .by_file
            .keys()
            .filter(|path: &&PathBuf| !path.exists())
            .cloned()
            .collect();
        for path in &gone {
            self.by_file.remove(path);
        }
        let sent: Vec<(PathBuf, Vec<usize>)> = self
            .by_file
            .iter()
            .map(|(path, lines): (&PathBuf, &BTreeSet<usize>)| {
                (path.clone(), lines.iter().copied().collect())
            })
            .collect();
        ForSession { sent, gone }
    }

    /// An edit at `anchor` added (`delta` > 0) or removed lines. Lines after
    /// the anchor move with their code; a removed line's breakpoint moves to
    /// the anchor.
    pub(crate) fn shift(&mut self, path: &Path, anchor: usize, delta: isize) {
        let Some(lines): Option<&mut BTreeSet<usize>> = self.by_file.get_mut(path) else {
            return;
        };
        let removed: usize = delta.unsigned_abs();
        let moved: BTreeSet<usize> = lines
            .iter()
            .map(|&line: &usize| {
                if line <= anchor {
                    line
                } else if delta >= 0 {
                    line.saturating_add(removed)
                } else if line <= anchor.saturating_add(removed) {
                    anchor
                } else {
                    line.saturating_sub(removed)
                }
            })
            .collect();
        *lines = moved;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path() -> PathBuf {
        PathBuf::from("/w/A.cs")
    }

    #[test]
    fn toggle_sets_then_clears_and_files_without_any_vanish() {
        let mut points: Breakpoints = Breakpoints::default();
        assert!(points.toggle(&path(), 4));
        assert!(points.has(&path(), 4));
        assert!(points.toggle(&path(), 9));
        assert_eq!(points.lines(&path()), [4, 9]);
        assert!(!points.toggle(&path(), 4));
        assert!(!points.toggle(&path(), 9));
        assert_eq!(points, Breakpoints::default(), "no empty file entries");
    }

    #[test]
    fn breakpoints_follow_their_code_when_lines_come_and_go_above() {
        let mut points: Breakpoints = Breakpoints::default();
        for line in [2, 5, 9] {
            points.toggle(&path(), line);
        }
        points.shift(&path(), 3, 2);
        assert_eq!(points.lines(&path()), [2, 7, 11], "two lines added at 3");
        points.shift(&path(), 2, 2);
        assert_eq!(
            points.lines(&path()),
            [2, 9, 13],
            "Enter on a breakpoint line keeps it"
        );
        points.shift(&path(), 8, -1);
        assert_eq!(points.lines(&path()), [2, 8, 12], "line 9 joined into 8");
        points.shift(&path(), 0, -20);
        assert_eq!(points.lines(&path()), [0], "a deleted block keeps one");
    }

    #[test]
    fn a_deleted_files_breakpoints_are_not_sent_and_are_dropped_and_named() {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let kept: PathBuf = dir.path().join("A.cs");
        let deleted: PathBuf = dir.path().join("B.cs");
        std::fs::write(&kept, "a").unwrap();
        std::fs::write(&deleted, "b").unwrap();
        let mut points: Breakpoints = Breakpoints::default();
        points.toggle(&kept, 3);
        points.toggle(&deleted, 5);
        std::fs::remove_file(&deleted).unwrap();
        let start: ForSession = points.for_session();
        assert_eq!(start.sent, vec![(kept.clone(), vec![3])]);
        assert_eq!(start.gone, vec![deleted.clone()]);
        assert_eq!(
            start.note().as_deref(),
            Some("Debug: breakpoints in B.cs dropped: the file no longer exists")
        );
        assert!(!points.has(&deleted, 5), "the dropped breakpoint is gone");
        assert_eq!(
            points.for_session().note(),
            None,
            "a clean start says nothing"
        );
    }
}
