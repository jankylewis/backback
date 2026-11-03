//! Breakpoints (BBEOD17): 0-based lines per file, toggled with F9 or a click
//! on the line number. They stay on their code when lines are added or
//! removed above them, as in VS Code and Rider.

mod bb_exceptions;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(crate) use bb_exceptions::{ExceptionBreaks, ExceptionKind};

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

    /// Every file with breakpoints and its lines, for a new session.
    pub(crate) fn all(&self) -> Vec<(PathBuf, Vec<usize>)> {
        self.by_file
            .iter()
            .map(|(path, lines): (&PathBuf, &BTreeSet<usize>)| {
                (path.clone(), lines.iter().copied().collect())
            })
            .collect()
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
}
