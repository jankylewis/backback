//! Editor groups side by side (BB-BUG-040), up to three (BB-BUG-045), as
//! VS Code's split editor. Every tab is in one list, tagged with its group,
//! the groups in order, so all code that walks every tab (the C# server,
//! rename, the disk watch, the quit prompt) still sees them all. A file is
//! open once: opening it again shows it in its group. Each group keeps its
//! active tab and its pane's scroll list; groups are numbered 0.. with no
//! gap (a group left empty closes and those right of it move left).

use std::path::PathBuf;

use gpui::UniformListScrollHandle;

use super::super::{Tab, Tabs};

/// The most groups side by side.
pub(crate) const MAX_GROUPS: usize = 3;

/// Which editor group a tab is in: 0 is the leftmost.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Group(pub(crate) usize);

impl Group {
    /// The group to the left, if any.
    pub(crate) fn left(self) -> Option<Self> {
        self.0.checked_sub(1).map(Self)
    }

    /// The group to the right (it may not exist yet).
    pub(crate) fn right(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

/// Each group's active file, by group number.
pub(super) type Actives = [Option<PathBuf>; MAX_GROUPS];

impl Tabs {
    /// How many groups show (1 with no tabs).
    pub(crate) fn groups(&self) -> usize {
        self.items
            .iter()
            .map(|tab: &Tab| tab.group.0.saturating_add(1))
            .max()
            .unwrap_or(1)
    }

    /// Whether more than one group shows.
    pub(crate) fn is_split(&self) -> bool {
        self.groups() > 1
    }

    pub(crate) fn focused(&self) -> Group {
        self.focused
    }

    /// The tabs of `group`, with their indices in the whole list.
    pub(crate) fn group(&self, group: Group) -> impl Iterator<Item = (usize, &Tab)> {
        self.items
            .iter()
            .enumerate()
            .filter(move |(_, tab): &(usize, &Tab)| tab.group == group)
    }

    pub(crate) fn active_of(&self, group: Group) -> Option<usize> {
        self.actives.get(group.0).copied().flatten()
    }

    pub(crate) fn active_tab_of(&self, group: Group) -> Option<&Tab> {
        self.active_of(group)
            .and_then(|index: usize| self.items.get(index))
    }

    /// The focused pane's line list.
    pub(crate) fn scroll(&self) -> &UniformListScrollHandle {
        self.scroll_of(self.focused)
    }

    /// `group`'s pane line list.
    pub(crate) fn scroll_of(&self, group: Group) -> &UniformListScrollHandle {
        self.scrolls.get(group.0).unwrap_or(&self.spare_scroll)
    }

    /// Focus `group`, if it has tabs.
    pub(crate) fn focus(&mut self, group: Group) {
        if self.group(group).next().is_some() {
            self.focused = group;
        }
    }

    /// Show tab `index` in its group and focus that group.
    pub(crate) fn switch(&mut self, index: usize) {
        if let Some(group) = self.items.get(index).map(|tab: &Tab| tab.group) {
            self.focused = group;
            self.set_active(group, Some(index));
        }
    }

    /// Cmd+`n`: the `n`-th tab (0-based) of the focused group.
    pub(crate) fn switch_nth(&mut self, n: usize) {
        let found: Option<usize> = self
            .group(self.focused)
            .nth(n)
            .map(|(index, _): (usize, &Tab)| index);
        if found.is_some() {
            self.set_active(self.focused, found);
        }
    }

    pub(in crate::editor::bb_tabs) fn set_active(&mut self, group: Group, index: Option<usize>) {
        if let Some(slot) = self.actives.get_mut(group.0) {
            *slot = index;
        }
    }

    /// Move tab `index` into `to` (a new group right of the last one is
    /// made, up to [`MAX_GROUPS`]), as a drop or Ctrl+Cmd+Arrow does; that
    /// group gets focus.
    pub(crate) fn move_to(&mut self, index: usize, to: Group) {
        let Some(from): Option<Group> = self.items.get(index).map(|tab: &Tab| tab.group) else {
            return;
        };
        let to: Group = Group(to.0.min(self.groups()).min(MAX_GROUPS.saturating_sub(1)));
        if from == to {
            return;
        }
        let mut actives: Actives = self.actives();
        // A move is no close: no Closed event, the server keeps the file.
        let mut tab: Tab = self.items.remove(index);
        let path: PathBuf = tab.path.clone();
        self.pass_on(&mut actives, index, from, &path);
        if self.group(to).next().is_none() {
            if let Some(scroll) = self.scrolls.get_mut(to.0) {
                *scroll = UniformListScrollHandle::default();
            }
        }
        tab.group = to;
        self.place(tab);
        if let Some(slot) = actives.get_mut(to.0) {
            *slot = Some(path);
        }
        self.restore(actives, to);
    }

    /// Close tab `index`; its group shows the next tab, or closes.
    pub(in crate::editor::bb_tabs) fn remove(&mut self, index: usize) {
        let Some(group): Option<Group> = self.items.get(index).map(|tab: &Tab| tab.group) else {
            return;
        };
        let mut actives: Actives = self.actives();
        let tab: Tab = self.items.remove(index);
        self.pass_on(&mut actives, index, group, &tab.path);
        self.events.push(super::super::TabEvent::Closed(tab.path));
        self.restore(actives, self.focused);
    }

    /// `path` left `group` from `index`: if it was the group's active file,
    /// its neighbour takes over.
    fn pass_on(&self, actives: &mut Actives, index: usize, group: Group, path: &std::path::Path) {
        let next: Option<PathBuf> = self.neighbour(index, group);
        if let Some(slot) = actives.get_mut(group.0) {
            if slot.as_deref() == Some(path) {
                *slot = next;
            }
        }
    }
}
