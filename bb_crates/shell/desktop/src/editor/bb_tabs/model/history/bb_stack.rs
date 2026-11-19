//! The two stacks behind Back and Forward. A place is reachable while its
//! tab is open or its file is on disk; a closed tab is reopened (VS Code),
//! a deleted or renamed file is skipped (BB-BUG-080).

use std::path::{Path, PathBuf};

use bb_text::LspPoint;

/// At most this many places to go back to.
pub(crate) const KEEP: usize = 50;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Place {
    pub(crate) path: PathBuf,
    pub(crate) at: LspPoint,
}

/// Where one Back or Forward press goes, and the files of the places it
/// dropped on the way because they are gone from disk.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct Hop {
    pub(crate) to: Option<Place>,
    pub(crate) gone: Vec<PathBuf>,
}

/// Is `path` open in a tab? Asked before the disk.
pub(crate) type IsOpen<'a> = &'a dyn Fn(&Path) -> bool;

#[derive(Debug, Default)]
pub(crate) struct History {
    back: Vec<Place>,
    forward: Vec<Place>,
}

impl History {
    /// A jump leaves `here`: remember it for Back; Forward is gone.
    pub(crate) fn leave(&mut self, here: Place) {
        if self.back.last() != Some(&here) {
            self.back.push(here);
            if self.back.len() > KEEP {
                self.back.remove(0);
            }
        }
        self.forward.clear();
    }

    /// Back has a place it can reach (the title bar's ← is live).
    pub(crate) fn can_back(&self, open: IsOpen<'_>) -> bool {
        reachable(&self.back, open)
    }

    /// Forward has a place it can reach.
    pub(crate) fn can_forward(&self, open: IsOpen<'_>) -> bool {
        reachable(&self.forward, open)
    }

    /// Where Back goes from `here` (none when no tab is open); `here`
    /// becomes Forward.
    pub(crate) fn back(&mut self, here: Option<Place>, open: IsOpen<'_>) -> Hop {
        hop((&mut self.back, &mut self.forward), here, open)
    }

    /// Where Forward goes from `here`; `here` becomes Back.
    pub(crate) fn forward(&mut self, here: Option<Place>, open: IsOpen<'_>) -> Hop {
        hop((&mut self.forward, &mut self.back), here, open)
    }
}

fn reachable(stack: &[Place], open: IsOpen<'_>) -> bool {
    stack.iter().any(|place: &Place| lives(&place.path, open))
}

/// Open in a tab, or a file on disk that a jump can reopen.
fn lives(path: &Path, open: IsOpen<'_>) -> bool {
    open(path) || path.is_file()
}

/// Pop `from` to its last reachable place, dropping the gone ones; `here`
/// (if a tab is open) goes onto `onto` only when the press goes somewhere.
fn hop(
    (from, onto): (&mut Vec<Place>, &mut Vec<Place>),
    here: Option<Place>,
    open: IsOpen<'_>,
) -> Hop {
    let mut gone: Vec<PathBuf> = Vec::new();
    while let Some(place) = from.pop() {
        if lives(&place.path, open) {
            onto.extend(here);
            return Hop {
                to: Some(place),
                gone,
            };
        }
        gone.push(place.path);
    }
    Hop { to: None, gone }
}
