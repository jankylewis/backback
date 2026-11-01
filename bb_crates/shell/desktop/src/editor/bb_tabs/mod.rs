use std::path::{Path, PathBuf};

use bb_text::{Document, DocumentError};

mod history;
mod tab;
mod window;

pub(crate) use history::{go_back, go_forward, GoBack, GoForward, History};
pub(crate) use tab::{Tab, TabError, TabEvent};

#[cfg(test)]
pub(crate) use window::LARGE_FILE_BYTES;

#[derive(Default)]
pub(crate) struct Tabs {
    items: Vec<Tab>,
    loading: Vec<PathBuf>,
    active: Option<usize>,
    pending_close: Option<usize>,
    events: Vec<TabEvent>,
    no_write: bool,
    /// Back / Forward across jumps.
    pub(crate) history: History,
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

    fn admit(&mut self, mut tab: Tab) {
        if self.no_write {
            tab.document.lock_writes();
        }
        self.events.push(TabEvent::Opened(tab.path.clone()));
        self.items.push(tab);
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
            self.active = Some(index);
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
        self.admit(loaded?);
        self.active = self.items.len().checked_sub(1);
        Ok(())
    }

    /// Add a tab loaded elsewhere (a rename opened it); the active tab stays.
    pub(crate) fn adopt(&mut self, tab: Tab) {
        self.admit(tab);
        if self.active.is_none() {
            self.active = Some(0);
        }
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

    pub(crate) fn switch(&mut self, index: usize) {
        if index < self.items.len() {
            self.active = Some(index);
        }
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
            self.pending_close = Some(index);
            return false;
        }
        self.remove(index);
        true
    }

    pub(crate) fn pending_close(&self) -> bool {
        self.pending_close.is_some()
    }

    pub(crate) fn request_window_close(&mut self) -> bool {
        self.pending_close = self
            .items
            .iter()
            .position(|tab: &Tab| tab.document.is_dirty());
        self.pending_close.is_none()
    }

    pub(crate) fn cancel_close(&mut self) {
        self.pending_close = None;
    }

    pub(crate) fn discard_close(&mut self) {
        if let Some(index) = self.pending_close.take() {
            self.remove(index);
        }
    }

    pub(crate) fn save_close(&mut self) -> Result<(), DocumentError> {
        if let Some(index) = self.pending_close {
            if let Some(tab) = self.items.get_mut(index) {
                tab.document.save()?;
            }
            self.pending_close = None;
            self.remove(index);
        }
        Ok(())
    }

    /// Opens and closes since the last call, oldest first.
    pub(crate) fn take_events(&mut self) -> Vec<TabEvent> {
        std::mem::take(&mut self.events)
    }

    fn remove(&mut self, index: usize) {
        let tab: Tab = self.items.remove(index);
        self.events.push(TabEvent::Closed(tab.path));
        self.active = match self.items.len() {
            0 => None,
            len => Some(index.min(len.saturating_sub(1))),
        };
    }
}

#[cfg(test)]
mod bb_tests;
