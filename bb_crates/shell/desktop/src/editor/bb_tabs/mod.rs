use std::path::{Path, PathBuf};

use bb_text::{Document, DocumentError};

mod bar;
mod model;
mod tab;

pub(crate) use bar::{
    apply as apply_tab_menu, bindings as tab_bindings, item as tab_menu_item, Item as TabMenuItem,
};
pub(crate) use bar::{bb_view, on_tab_actions, TabMenu};
pub(crate) use model::bb_groups::Side;
pub(crate) use model::history::{go_back, go_forward, GoBack, GoForward, History};
pub(crate) use tab::{Tab, TabError, TabEvent};

#[cfg(test)]
pub(crate) use tab::LARGE_FILE_BYTES;

#[derive(Default)]
pub(crate) struct Tabs {
    items: Vec<Tab>,
    loading: Vec<PathBuf>,
    /// The focused group's active tab; `other_active` the other group's.
    active: Option<usize>,
    other_active: Option<usize>,
    focused: Side,
    /// The focused pane's line list; `other_scroll` the other pane's.
    pub(crate) scroll: gpui::UniformListScrollHandle,
    other_scroll: gpui::UniformListScrollHandle,
    /// The left pane's width while split, set as the window draws.
    pub(crate) pane_px: f32,
    /// The file the save prompt asks about: by path, so a tab that moves
    /// while it asks is still the one saved or discarded (BB-BUG-041).
    pending_close: Option<PathBuf>,
    /// Tabs with unsaved edits a bulk close still has to ask about.
    closing: Vec<PathBuf>,
    /// The open right-click menu of a tab.
    pub(crate) menu: Option<TabMenu>,
    events: Vec<TabEvent>,
    no_write: bool,
    /// Back / Forward across jumps.
    pub(crate) history: History,
    /// The last whole-line copy or cut (BB-BUG-028): pasted with no
    /// selection, it goes in above the cursor's line.
    pub(crate) line_clip: Option<String>,
    /// The file the Explorer last opened its folders to (BB-BUG-029).
    pub(crate) revealed: Option<PathBuf>,
}

impl Tabs {
    /// Lock saves in every tab, now and later (readback on real solutions).
    pub(crate) fn set_no_write(&mut self) {
        self.no_write = true;
        for tab in &mut self.items {
            tab.document.lock_writes();
        }
    }

    /// Whether every save is locked.
    pub(crate) fn no_write(&self) -> bool {
        self.no_write
    }

    /// Add `tab` at the end of the focused group; its index.
    fn admit(&mut self, mut tab: Tab) -> usize {
        if self.no_write {
            tab.document.lock_writes();
        }
        self.events.push(TabEvent::Opened(tab.path.clone()));
        tab.side = self.focused;
        let actives: (Option<PathBuf>, Option<PathBuf>) = self.actives();
        let at: usize = self.place(tab);
        self.restore(actives, self.focused);
        at
    }

    /// Start opening `path`. `Ok(None)`: it is already open (now active) or
    /// loading. `Ok(Some(file))`: load `file` with [`Tab::load`] off the UI
    /// thread, then call [`Tabs::finish_open`].
    pub(crate) fn begin_open(&mut self, path: &Path) -> Result<Option<PathBuf>, TabError> {
        let canonical: PathBuf = path.canonicalize()?;
        if let Some(index) = self
            .items
            .iter()
            .position(|tab: &Tab| tab.path == canonical)
        {
            // Open once: show it in its own group (BB-BUG-040).
            self.switch(index);
            return Ok(None);
        }
        if self.loading.contains(&canonical) {
            return Ok(None);
        }
        self.loading.push(canonical.clone());
        Ok(Some(canonical))
    }

    /// Finish a load started by [`Tabs::begin_open`]; the new tab becomes active.
    pub(crate) fn finish_open(
        &mut self,
        path: &Path,
        loaded: Result<Tab, TabError>,
    ) -> Result<(), TabError> {
        self.loading.retain(|pending: &PathBuf| pending != path);
        let at: usize = self.admit(loaded?);
        self.switch(at);
        Ok(())
    }

    /// Add a tab loaded elsewhere (a rename opened it); the active tab stays.
    pub(crate) fn adopt(&mut self, tab: Tab) {
        let _at: usize = self.admit(tab);
    }

    /// Files still loading, shown as placeholder tabs.
    pub(crate) fn loading(&self) -> &[PathBuf] {
        &self.loading
    }

    #[cfg(test)]
    pub(crate) fn open(&mut self, path: &Path) -> Result<(), TabError> {
        match self.begin_open(path)? {
            Some(file) => self.finish_open(&file.clone(), Tab::load(file)),
            None => Ok(()),
        }
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.items.len()
    }

    pub(crate) fn active_index(&self) -> Option<usize> {
        self.active
    }

    pub(crate) fn active(&self) -> Option<&Document> {
        self.active
            .and_then(|index: usize| self.items.get(index))
            .map(|tab: &Tab| &tab.document)
    }

    pub(crate) fn active_tab(&self) -> Option<&Tab> {
        self.active.and_then(|index: usize| self.items.get(index))
    }

    pub(crate) fn tab_mut(&mut self, path: &Path) -> Option<&mut Tab> {
        self.items
            .iter_mut()
            .find(|tab: &&mut Tab| tab.path == path)
    }

    pub(crate) fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.active
            .and_then(|index: usize| self.items.get_mut(index))
    }

    pub(crate) fn active_mut(&mut self) -> Option<&mut Document> {
        self.active
            .and_then(|index: usize| self.items.get_mut(index))
            .map(|tab: &mut Tab| &mut tab.document)
    }

    pub(crate) fn items(&self) -> &[Tab] {
        &self.items
    }

    pub(crate) fn items_mut(&mut self) -> &mut [Tab] {
        &mut self.items
    }

    pub(crate) fn close_active(&mut self) -> bool {
        let index: usize = match self.active {
            Some(index) => index,
            None => return true,
        };
        self.close(index)
    }

    pub(crate) fn close(&mut self, index: usize) -> bool {
        let tab: &Tab = match self.items.get(index) {
            Some(tab) => tab,
            None => return true,
        };
        if tab.document.is_dirty() {
            self.pending_close = Some(tab.path.clone());
            return false;
        }
        self.remove(index);
        true
    }

    pub(crate) fn pending_close(&self) -> bool {
        self.pending_close.is_some()
    }

    /// The file the save prompt asks about.
    pub(crate) fn asking(&self) -> Option<&Path> {
        self.pending_close.as_deref()
    }

    pub(crate) fn request_window_close(&mut self) -> bool {
        self.pending_close = self
            .items
            .iter()
            .find(|tab: &&Tab| tab.document.is_dirty())
            .map(|tab: &Tab| tab.path.clone());
        self.pending_close.is_none()
    }

    pub(crate) fn cancel_close(&mut self) {
        self.pending_close = None;
        self.closing.clear();
    }

    pub(crate) fn discard_close(&mut self) {
        if let Some(path) = self.pending_close.take() {
            if let Some(index) = self.position(&path) {
                self.remove(index);
            }
            let _asked: bool = self.next_close();
        }
    }

    pub(crate) fn save_close(&mut self) -> Result<(), DocumentError> {
        let Some(path): Option<PathBuf> = self.pending_close.clone() else {
            return Ok(());
        };
        if let Some(tab) = self.tab_mut(&path) {
            tab.document.save()?;
        }
        self.pending_close = None;
        if let Some(index) = self.position(&path) {
            self.remove(index);
        }
        let _asked: bool = self.next_close();
        Ok(())
    }

    /// Opens and closes since the last call, oldest first.
    pub(crate) fn take_events(&mut self) -> Vec<TabEvent> {
        std::mem::take(&mut self.events)
    }
}

#[cfg(test)]
mod bb_tests;
