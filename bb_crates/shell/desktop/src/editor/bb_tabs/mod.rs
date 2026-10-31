use std::path::{Path, PathBuf};
use std::time::Instant;

use bb_syntax::{ByteEdit, CSharpSyntax, SyntaxError, Tree};
use bb_text::{Document, DocumentError, TextChange};

pub(crate) struct Tab {
    pub(crate) path: PathBuf,
    pub(crate) document: Document,
    pub(crate) syntax: CSharpSyntax,
    pub(crate) tree: Tree,
    pub(crate) parsed_text: String,
    pub(crate) line_bytes: Vec<usize>,
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
    fn new(path: PathBuf) -> Result<Self, TabError> {
        let started: Instant = Instant::now();
        let document: Document = Document::open(&path)?;
        let parsed_text: String = document.text();
        let mut syntax: CSharpSyntax = CSharpSyntax::new()?;
        let tree: Tree = syntax.parse(&parsed_text, None)?;
        let line_bytes: Vec<usize> = line_starts(&parsed_text);
        eprintln!(
            "editor_open_ms={:.3}",
            started.elapsed().as_secs_f64() * 1000.0
        );
        Ok(Self {
            path,
            document,
            syntax,
            tree,
            parsed_text,
            line_bytes,
        })
    }

    pub(crate) fn reparse(&mut self) -> Result<(), TabError> {
        let after: String = self.document.text();
        if after != self.parsed_text {
            let change: Option<TextChange> = self.document.last_change();
            self.tree = if let Some(change) = change {
                let start: usize = char_byte(&self.parsed_text, change.at.raw());
                let old_end: usize = char_byte(
                    &self.parsed_text,
                    change.at.raw().saturating_add(change.removed_chars),
                );
                let edit: ByteEdit = ByteEdit {
                    start,
                    old_end,
                    new_end: start.saturating_add(change.inserted_bytes),
                };
                self.syntax
                    .reparse_bytes(&self.tree, &self.parsed_text, &after, edit)?
            } else {
                self.syntax.reparse(&self.tree, &self.parsed_text, &after)?
            };
            self.line_bytes = line_starts(&after);
            self.parsed_text = after;
        }
        Ok(())
    }
}

fn char_byte(text: &str, index: usize) -> usize {
    text.char_indices()
        .nth(index)
        .map_or(text.len(), |(byte, _): (usize, char)| byte)
}

fn line_starts(text: &str) -> Vec<usize> {
    let mut starts: Vec<usize> = vec![0];
    for (index, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(index.saturating_add(1));
        }
    }
    starts
}

#[derive(Default)]
pub(crate) struct Tabs {
    items: Vec<Tab>,
    active: Option<usize>,
    pending_close: Option<usize>,
}

impl Tabs {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn open(&mut self, path: &Path) -> Result<(), TabError> {
        let canonical: PathBuf = path.canonicalize()?;
        if let Some(index) = self
            .items
            .iter()
            .position(|tab: &Tab| tab.path == canonical)
        {
            self.active = Some(index);
            return Ok(());
        }
        self.items.push(Tab::new(canonical)?);
        self.active = self.items.len().checked_sub(1);
        Ok(())
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
