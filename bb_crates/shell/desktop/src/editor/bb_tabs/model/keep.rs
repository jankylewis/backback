//! The bookkeeping every group change shares (BB-BUG-040): each group's
//! active file kept by path across a change, where a tab goes in its run,
//! and the rules that hold after any change ([`Tabs::settle`]).

use std::path::{Path, PathBuf};

use super::super::{Tab, Tabs};
use super::bb_groups::{Actives, Side};

/// The bookkeeping the group changes share.
impl Tabs {
    fn active_path(&self, side: Side) -> Option<PathBuf> {
        self.active_tab_of(side).map(|tab: &Tab| tab.path.clone())
    }

    pub(in crate::editor::bb_tabs) fn actives(&self) -> Actives {
        (self.active_path(Side::Left), self.active_path(Side::Right))
    }

    fn find(&self, path: Option<PathBuf>) -> Option<usize> {
        path.and_then(|path: PathBuf| self.position(&path))
    }

    /// Set each group's active file and the focus, then [`Self::settle`].
    pub(in crate::editor::bb_tabs) fn restore(&mut self, (left, right): Actives, focus: Side) {
        if focus != self.focused {
            std::mem::swap(&mut self.scroll, &mut self.other_scroll);
            self.focused = focus;
        }
        let (focused, other): (Option<PathBuf>, Option<PathBuf>) = match focus {
            Side::Left => (left, right),
            Side::Right => (right, left),
        };
        (self.active, self.other_active) = (self.find(focused), self.find(other));
        self.settle();
    }

    /// Tab `at`'s file, if it is in `side`'s group.
    fn path_in(&self, at: usize, side: Side) -> Option<PathBuf> {
        self.items
            .get(at)
            .filter(|tab: &&Tab| tab.side == side)
            .map(|tab: &Tab| tab.path.clone())
    }

    /// The tab of `side` that takes the place of a removed one at `index`.
    pub(in crate::editor::bb_tabs) fn neighbour(
        &self,
        index: usize,
        side: Side,
    ) -> Option<PathBuf> {
        self.path_in(index, side).or_else(|| {
            index
                .checked_sub(1)
                .and_then(|at: usize| self.path_in(at, side))
        })
    }

    /// Where `side`'s run starts in the list.
    fn start_of(&self, side: Side) -> usize {
        match side {
            Side::Left => 0,
            Side::Right => self.group(Side::Left).count(),
        }
    }

    /// Put `tab` in its side's run: a pinned one at the end of the pinned
    /// tabs, another at the end. Its index.
    pub(in crate::editor::bb_tabs) fn place(&mut self, tab: Tab) -> usize {
        let at: usize = if tab.pinned {
            self.pins_end(tab.side)
        } else {
            self.start_of(tab.side)
                .saturating_add(self.group(tab.side).count())
        };
        self.items.insert(at, tab);
        at
    }

    /// Where `side`'s pinned run ends: its first unpinned tab's index.
    pub(in crate::editor::bb_tabs) fn pins_end(&self, side: Side) -> usize {
        self.start_of(side).saturating_add(
            self.group(side)
                .filter(|(_, t): &(usize, &Tab)| t.pinned)
                .count(),
        )
    }

    pub(in crate::editor::bb_tabs) fn position(&self, path: &Path) -> Option<usize> {
        self.items.iter().position(|tab: &Tab| tab.path == path)
    }

    /// `at` if it is a tab of `side`, else that group's first tab.
    fn own(&self, at: Option<usize>, side: Side) -> Option<usize> {
        at.filter(|i: &usize| self.path_in(*i, side).is_some())
            .or_else(|| self.group(side).next().map(|(i, _): (usize, &Tab)| i))
    }

    /// Keep the rules after a change: a group without tabs is gone (the
    /// right group's tabs become the left group), the focused group has tabs
    /// when any does, and each group's active tab is one of its own.
    fn settle(&mut self) {
        if self.group(Side::Left).next().is_none() && self.is_split() {
            for tab in &mut self.items {
                tab.side = Side::Left;
            }
            if self.focused == Side::Left {
                std::mem::swap(&mut self.scroll, &mut self.other_scroll);
                self.active = self.other_active;
            }
            self.focused = Side::Left;
        }
        if self.group(self.focused).next().is_none() {
            self.focus(self.focused.other());
        }
        self.active = self.own(self.active, self.focused);
        self.other_active = self.own(self.other_active, self.focused.other());
    }
}
