//! Pinned tabs and the bulk closes of a tab's menu (BB-BUG-039), as VS Code
//! has them: pinned tabs stay left; Close Others / to the Right / to the
//! Left and Close All Except Pinned skip them; Close All takes every tab.

use std::path::{Path, PathBuf};

use super::{Tab, Tabs};

pub(crate) mod bb_view;
mod menu;

pub(crate) use menu::{apply, bindings, item, on_tab_actions, Item};

/// Which tabs a bulk close takes, seen from the tab whose menu it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CloseSet {
    /// Every unpinned tab but this one.
    Others,
    /// Unpinned tabs after this one.
    Right,
    /// Unpinned tabs before this one.
    Left,
    /// Every unpinned tab.
    Unpinned,
    /// Every tab, pinned or not.
    All,
}

impl CloseSet {
    fn takes(self, at: usize, index: usize, tab: &Tab) -> bool {
        match self {
            Self::Others => index != at && !tab.pinned,
            Self::Right => index > at && !tab.pinned,
            Self::Left => index < at && !tab.pinned,
            Self::Unpinned => !tab.pinned,
            Self::All => true,
        }
    }
}

/// The open tab menu: which tab, and where the right-click was.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TabMenu {
    pub(crate) index: usize,
    pub(crate) at: gpui::Point<gpui::Pixels>,
}

impl Tabs {
    fn position(&self, path: &Path) -> Option<usize> {
        self.items.iter().position(|tab: &Tab| tab.path == path)
    }

    /// Pin or unpin tab `index`: a pin moves it to the end of the pinned
    /// run, an unpin to just after it. The active tab stays the same file.
    pub(crate) fn toggle_pin(&mut self, index: usize) {
        if index >= self.items.len() {
            return;
        }
        let active: Option<PathBuf> = self.active_tab().map(|tab: &Tab| tab.path.clone());
        let mut tab: Tab = self.items.remove(index);
        tab.pinned = !tab.pinned;
        let at: usize = self.items.iter().take_while(|t: &&Tab| t.pinned).count();
        self.items.insert(at, tab);
        self.active = active.and_then(|path: PathBuf| self.position(&path));
    }

    /// Close the tabs `set` takes, seen from tab `at`. Clean tabs close now;
    /// tabs with unsaved edits queue for the save prompt, one at a time.
    /// The menu's tab, if still open, becomes active. `true`: no prompt.
    pub(crate) fn close_set(&mut self, at: usize, set: CloseSet) -> bool {
        let anchor: Option<PathBuf> = self.items.get(at).map(|tab: &Tab| tab.path.clone());
        let taken: Vec<PathBuf> = self
            .items
            .iter()
            .enumerate()
            .filter(|(index, tab): &(usize, &Tab)| set.takes(at, *index, tab))
            .map(|(_, tab): (usize, &Tab)| tab.path.clone())
            .collect();
        for path in taken {
            let Some(index): Option<usize> = self.position(&path) else {
                continue;
            };
            if self
                .items
                .get(index)
                .is_some_and(|tab: &Tab| tab.document.is_dirty())
            {
                self.closing.push(path);
            } else {
                self.remove(index);
            }
        }
        if let Some(index) = anchor.and_then(|path: PathBuf| self.position(&path)) {
            self.active = Some(index);
        }
        self.next_close()
    }

    /// Show the next queued tab with unsaved edits and ask about it;
    /// `true` when none is left.
    pub(super) fn next_close(&mut self) -> bool {
        while !self.closing.is_empty() {
            let path: PathBuf = self.closing.remove(0);
            if let Some(index) = self.position(&path) {
                self.pending_close = Some(index);
                self.active = Some(index);
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod window_tests;
