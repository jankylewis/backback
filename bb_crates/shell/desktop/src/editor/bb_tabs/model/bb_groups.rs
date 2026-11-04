//! Two editor groups side by side (BB-BUG-040), as VS Code's split editor.
//! Every tab is in one list, tagged with its side, the left tabs first, so
//! all code that walks every tab (the C# server, rename, the disk watch,
//! the quit prompt) still sees them all. A file is open once: opening it
//! again shows it in its group. `active` is the focused group's tab and
//! `other_active` the other group's; `scroll` is the focused pane's list.

use std::path::PathBuf;

use gpui::UniformListScrollHandle;

use super::super::{Tab, Tabs};

/// Which group a tab is in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Side {
    #[default]
    Left,
    Right,
}

impl Side {
    pub(crate) fn other(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

/// Each group's active file, left then right.
pub(super) type Actives = (Option<PathBuf>, Option<PathBuf>);

impl Tabs {
    /// Whether a right group shows.
    pub(crate) fn is_split(&self) -> bool {
        self.items.iter().any(|tab: &Tab| tab.side == Side::Right)
    }

    pub(crate) fn focused(&self) -> Side {
        self.focused
    }

    /// The tabs of `side`, with their indices in the whole list.
    pub(crate) fn group(&self, side: Side) -> impl Iterator<Item = (usize, &Tab)> {
        self.items
            .iter()
            .enumerate()
            .filter(move |(_, tab): &(usize, &Tab)| tab.side == side)
    }

    pub(crate) fn active_of(&self, side: Side) -> Option<usize> {
        if side == self.focused {
            self.active
        } else {
            self.other_active
        }
    }

    pub(crate) fn active_tab_of(&self, side: Side) -> Option<&Tab> {
        self.active_of(side)
            .and_then(|index: usize| self.items.get(index))
    }

    /// The pane list of the group that is not focused.
    pub(crate) fn other_scroll(&self) -> &UniformListScrollHandle {
        &self.other_scroll
    }

    /// Focus `side`'s group, if it has tabs.
    pub(crate) fn focus(&mut self, side: Side) {
        if side == self.focused || self.group(side).next().is_none() {
            return;
        }
        std::mem::swap(&mut self.active, &mut self.other_active);
        std::mem::swap(&mut self.scroll, &mut self.other_scroll);
        self.focused = side;
    }

    /// Show tab `index` in its group and focus that group.
    pub(crate) fn switch(&mut self, index: usize) {
        if let Some(side) = self.items.get(index).map(|tab: &Tab| tab.side) {
            self.focus(side);
            self.active = Some(index);
        }
    }

    /// Cmd+`n`: the `n`-th tab (0-based) of the focused group.
    pub(crate) fn switch_nth(&mut self, n: usize) {
        let found: Option<usize> = self
            .group(self.focused)
            .nth(n)
            .map(|(index, _): (usize, &Tab)| index);
        if let Some(index) = found {
            self.active = Some(index);
        }
    }

    /// Move tab `index` into `side`'s group (made if it does not exist yet),
    /// as a drop or Ctrl+Cmd+Arrow does; that group gets focus.
    pub(crate) fn move_to(&mut self, index: usize, side: Side) {
        let Some(from): Option<Side> = self.items.get(index).map(|tab: &Tab| tab.side) else {
            return;
        };
        if from == side {
            return;
        }
        let (mut left, mut right): Actives = self.actives();
        // A move is no close: no Closed event, the server keeps the file.
        let mut tab: Tab = self.items.remove(index);
        let path: PathBuf = tab.path.clone();
        let next: Option<PathBuf> = self.neighbour(index, from);
        let source: &mut Option<PathBuf> = if from == Side::Left {
            &mut left
        } else {
            &mut right
        };
        if source.as_deref() == Some(path.as_path()) {
            *source = next;
        }
        if !self.is_split() {
            self.other_scroll = UniformListScrollHandle::default();
        }
        tab.side = side;
        self.place(tab);
        if side == Side::Left {
            left = Some(path);
        } else {
            right = Some(path);
        }
        self.restore((left, right), side);
    }

    /// Close tab `index`; its group shows the next tab, or goes.
    pub(in crate::editor::bb_tabs) fn remove(&mut self, index: usize) {
        let Some(side): Option<Side> = self.items.get(index).map(|tab: &Tab| tab.side) else {
            return;
        };
        let (mut left, mut right): (Option<PathBuf>, Option<PathBuf>) = self.actives();
        let tab: Tab = self.items.remove(index);
        let next: Option<PathBuf> = self.neighbour(index, side);
        let slot: &mut Option<PathBuf> = if side == Side::Left {
            &mut left
        } else {
            &mut right
        };
        if slot.as_deref() == Some(tab.path.as_path()) {
            *slot = next;
        }
        self.events.push(super::super::TabEvent::Closed(tab.path));
        self.restore((left, right), self.focused);
    }
}
