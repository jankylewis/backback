//! The bookkeeping every group change shares (BB-BUG-040, BB-BUG-045):
//! each group's active file kept by path across a change, where a tab goes
//! in its group's run, and the rules that hold after any change
//! ([`Tabs::settle`]).

use std::path::{Path, PathBuf};

use gpui::UniformListScrollHandle;

use super::super::{Tab, Tabs};
use super::bb_groups::{Actives, Group, MAX_GROUPS};

impl Tabs {
    pub(in crate::editor::bb_tabs) fn actives(&self) -> Actives {
        std::array::from_fn(|g: usize| {
            self.active_tab_of(Group(g))
                .map(|tab: &Tab| tab.path.clone())
        })
    }

    /// Set each group's active file and the focus, then [`Self::settle`].
    pub(in crate::editor::bb_tabs) fn restore(&mut self, actives: Actives, focus: Group) {
        self.focused = focus;
        for (g, path) in actives.into_iter().enumerate() {
            let at: Option<usize> = path.and_then(|path: PathBuf| self.position(&path));
            self.set_active(Group(g), at);
        }
        self.settle();
    }

    /// Tab `at`'s file, if it is in `group`.
    fn path_in(&self, at: usize, group: Group) -> Option<PathBuf> {
        self.items
            .get(at)
            .filter(|tab: &&Tab| tab.group == group)
            .map(|tab: &Tab| tab.path.clone())
    }

    /// The tab of `group` that takes the place of a removed one at `index`.
    pub(in crate::editor::bb_tabs) fn neighbour(
        &self,
        index: usize,
        group: Group,
    ) -> Option<PathBuf> {
        self.path_in(index, group).or_else(|| {
            index
                .checked_sub(1)
                .and_then(|at: usize| self.path_in(at, group))
        })
    }

    /// Where `group`'s run starts in the list.
    fn start_of(&self, group: Group) -> usize {
        self.items
            .iter()
            .filter(|tab: &&Tab| tab.group < group)
            .count()
    }

    /// Put `tab` in its group's run: a pinned one at the end of the pinned
    /// tabs, another at the end. Its index.
    pub(in crate::editor::bb_tabs) fn place(&mut self, tab: Tab) -> usize {
        let at: usize = if tab.pinned {
            self.pins_end(tab.group)
        } else {
            self.start_of(tab.group)
                .saturating_add(self.group(tab.group).count())
        };
        self.items.insert(at, tab);
        at
    }

    /// Where `group`'s pinned run ends: its first unpinned tab's index.
    pub(in crate::editor::bb_tabs) fn pins_end(&self, group: Group) -> usize {
        self.start_of(group).saturating_add(
            self.group(group)
                .filter(|(_, t): &(usize, &Tab)| t.pinned)
                .count(),
        )
    }

    pub(in crate::editor::bb_tabs) fn position(&self, path: &Path) -> Option<usize> {
        self.items.iter().position(|tab: &Tab| tab.path == path)
    }

    /// `at` if it is a tab of `group`, else that group's first tab.
    fn own(&self, at: Option<usize>, group: Group) -> Option<usize> {
        at.filter(|i: &usize| self.path_in(*i, group).is_some())
            .or_else(|| self.group(group).next().map(|(i, _): (usize, &Tab)| i))
    }

    /// Keep the rules after a change: groups are numbered 0.. with no gap
    /// (an empty group closes; those right of it move left, with their
    /// active tab and scroll list); the focused group has tabs when any
    /// does; each group's active tab is one of its own.
    fn settle(&mut self) {
        let present: Vec<usize> = (0..MAX_GROUPS)
            .filter(|g: &usize| self.group(Group(*g)).next().is_some())
            .collect();
        let old_actives: [Option<usize>; MAX_GROUPS] = self.actives;
        let old_scrolls: [UniformListScrollHandle; MAX_GROUPS] = self.scrolls.clone();
        let mut focused: usize = self.focused.0;
        if !present.contains(&focused) {
            // The group that takes its place: the one right of it, else left.
            focused = present
                .iter()
                .position(|g: &usize| *g > focused)
                .or_else(|| present.len().checked_sub(1))
                .and_then(|new: usize| present.get(new).copied())
                .unwrap_or(0);
        }
        self.actives = [None; MAX_GROUPS];
        self.scrolls = std::array::from_fn(|_: usize| UniformListScrollHandle::default());
        for (new, old) in present.iter().enumerate() {
            for tab in self
                .items
                .iter_mut()
                .filter(|tab: &&mut Tab| tab.group.0 == *old)
            {
                tab.group = Group(new);
            }
            if let Some(slot) = self.actives.get_mut(new) {
                *slot = old_actives.get(*old).copied().flatten();
            }
            if let Some(slot) = self.scrolls.get_mut(new) {
                *slot = old_scrolls.get(*old).cloned().unwrap_or_default();
            }
            if *old == focused {
                self.focused = Group(new);
            }
        }
        if present.is_empty() {
            self.focused = Group(0);
        }
        for g in 0..MAX_GROUPS {
            let at: Option<usize> = self.own(self.active_of(Group(g)), Group(g));
            self.set_active(Group(g), at);
        }
    }
}
