//! Popup and panel state. Pure data: the view draws it, the keys move it.

use std::collections::BTreeMap;
use std::path::PathBuf;

use bb_intel::{ActionItem, CompletionEntry, FileChange};
use bb_text::{CharIdx, LspPoint};

use crate::intel::bb_worker::protocol::Place;

/// A list with one selected row that wraps neither way.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Pick {
    /// The selected row.
    pub(crate) selected: usize,
}

impl Pick {
    /// Move the selection down (or up) inside `len` rows.
    pub(crate) fn step(&mut self, down: bool, len: usize) {
        self.selected = if down {
            self.selected.saturating_add(1).min(len.saturating_sub(1))
        } else {
            self.selected.saturating_sub(1)
        };
    }
}

/// The completion list, filtered by what was typed after the anchor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CompletionPopup {
    /// The document.
    pub(crate) path: PathBuf,
    /// Where the word being completed starts.
    pub(crate) anchor: CharIdx,
    items: Vec<CompletionEntry>,
    filter: String,
    /// The selected visible row.
    pub(crate) pick: Pick,
    /// The UI frame the popup was first drawn in.
    pub(crate) drawn_frame: Option<u64>,
}

impl CompletionPopup {
    pub(crate) fn new(path: PathBuf, anchor: CharIdx, items: Vec<CompletionEntry>) -> Self {
        Self {
            path,
            anchor,
            items,
            filter: String::new(),
            pick: Pick::default(),
            drawn_frame: None,
        }
    }

    /// Filter by `typed` (the text from the anchor to the cursor).
    pub(crate) fn set_filter(&mut self, typed: &str) {
        if typed != self.filter {
            typed.clone_into(&mut self.filter);
            self.pick = Pick::default();
        }
    }

    /// Items whose label starts with the filter (any case), then items that
    /// contain it, each group in server order.
    pub(crate) fn visible(&self) -> Vec<&CompletionEntry> {
        let wanted: String = self.filter.to_lowercase();
        let (mut starts, mut contains): (Vec<&CompletionEntry>, Vec<&CompletionEntry>) =
            (Vec::new(), Vec::new());
        for item in &self.items {
            let label: String = item.label.to_lowercase();
            if label.starts_with(&wanted) {
                starts.push(item);
            } else if label.contains(&wanted) {
                contains.push(item);
            }
        }
        starts.append(&mut contains);
        starts
    }

    /// Move the selection.
    pub(crate) fn step(&mut self, down: bool) {
        let len: usize = self.visible().len();
        self.pick.step(down, len);
    }

    /// The label Enter would insert.
    pub(crate) fn chosen(&self) -> Option<String> {
        self.visible()
            .get(self.pick.selected)
            .map(|item: &&CompletionEntry| item.label.clone())
    }
}

/// Hover text near the point it is about.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HoverPopup {
    /// The text.
    pub(crate) text: String,
    /// The line it is about.
    pub(crate) line: u32,
    /// Window position to draw at.
    pub(crate) x: f32,
    /// Window position to draw at.
    pub(crate) y: f32,
}

/// The code actions offered at the cursor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ActionsPopup {
    /// The actions, in server order.
    pub(crate) items: Vec<ActionItem>,
    /// The selected row.
    pub(crate) pick: Pick,
}

/// A panel of places (references), each a click target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PlacesPanel {
    /// What the panel shows, such as `3 references`.
    pub(crate) title: String,
    /// The places.
    pub(crate) places: Vec<Place>,
    /// The selected row.
    pub(crate) pick: Pick,
}

/// The new-name box F2 opens.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RenameInput {
    /// The document.
    pub(crate) path: PathBuf,
    /// The symbol's position.
    pub(crate) at: LspPoint,
    /// The name typed so far.
    pub(crate) text: String,
}

/// A rename plan shown before anything changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RenamePreview {
    /// The new name.
    pub(crate) name: String,
    /// The server's plan.
    pub(crate) changes: Vec<FileChange>,
    /// The revision of every open tab when the plan was asked for.
    pub(crate) revisions: BTreeMap<PathBuf, u64>,
}

impl RenamePreview {
    /// Files and text edits in the plan.
    pub(crate) fn counts(&self) -> (usize, usize) {
        let edits: usize = self
            .changes
            .iter()
            .map(|change: &FileChange| match change {
                FileChange::Edit { changes, .. } => changes.len(),
                FileChange::Create(_) | FileChange::Rename { .. } | FileChange::Delete(_) => 1,
            })
            .sum();
        (self.changes.len(), edits)
    }
}
