use std::path::{Path, PathBuf};
use std::time::Instant;

use bb_syntax::{CSharpSyntax, LiveTree, SyntaxError, Tree};
use bb_text::{Document, DocumentError, TextChange};

mod window;

#[cfg(test)]
pub(crate) use window::LARGE_FILE_BYTES;

pub(crate) struct Tab {
    pub(crate) path: PathBuf,
    pub(crate) document: Document,
    /// Highlight queries; parsing uses the parser inside `live`.
    pub(crate) highlighter: CSharpSyntax,
    pub(crate) live: LiveTree,
    /// Lines the tree covers; `None` for a file parsed whole.
    pub(crate) window_lines: Option<std::ops::Range<usize>>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum TabError {
    #[error("document: {0}")]
    Document(#[from] DocumentError),
    #[error("syntax: {0}")]
    Syntax(#[from] SyntaxError),
    #[error("file path: {0}")]
    Path(#[from] std::io::Error),
}

impl Tab {
    /// Read and parse a file. Slow on a big file: call it off the UI thread.
    pub(crate) fn load(path: PathBuf) -> Result<Self, TabError> {
        let started: Instant = Instant::now();
        let document: Document = Document::open(&path)?;
        let window_lines: Option<std::ops::Range<usize>> = window::first_window(&document);
        let mut syntax: CSharpSyntax = CSharpSyntax::new()?;
        let tree: Tree = syntax.parse_rope_in(
            document.rope(),
            None,
            window::points(&document, window_lines.as_ref()),
        )?;
        let highlighter: CSharpSyntax = CSharpSyntax::new()?;
        eprintln!(
            "editor_open_ms={:.3}",
            started.elapsed().as_secs_f64() * 1000.0
        );
        Ok(Self {
            path,
            document,
            highlighter,
            live: LiveTree::new(syntax, tree),
            window_lines,
        })
    }

    /// Move the document's new edits onto the shown tree. O(depth) per edit;
    /// the parse itself runs later, off the UI thread.
    pub(crate) fn sync_edits(&mut self) {
        let changes: Vec<TextChange> = self.document.take_changes();
        self.live.edit(&changes);
    }

    /// Sync and parse on this thread, as a test oracle for the background path.
    #[cfg(test)]
    pub(crate) fn reparse(&mut self) -> Result<(), TabError> {
        self.sync_edits();
        self.live
            .parse_now(self.document.rope(), self.parse_window())?;
        Ok(())
    }
}

#[derive(Default)]
pub(crate) struct Tabs {
    items: Vec<Tab>,
    loading: Vec<PathBuf>,
    active: Option<usize>,
    pending_close: Option<usize>,
}

impl Tabs {
    pub(crate) fn new() -> Self {
        Self::default()
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
        self.items.push(loaded?);
        self.active = self.items.len().checked_sub(1);
        Ok(())
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

    fn remove(&mut self, index: usize) {
        self.items.remove(index);
        self.active = match self.items.len() {
            0 => None,
            len => Some(index.min(len.saturating_sub(1))),
        };
    }
}

#[cfg(test)]
mod bb_tests;
