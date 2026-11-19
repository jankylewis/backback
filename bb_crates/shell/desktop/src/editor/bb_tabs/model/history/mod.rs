//! Back and Forward (Cmd+[ / Cmd+], Ctrl+- / Ctrl+Shift+-): the places a
//! jump left behind, as in Rider and VS Code. A new jump clears Forward.

mod bb_stack;
#[cfg(test)]
mod family_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod window_tests;

use std::path::Path;

use gpui::{Context, Window};

pub(crate) use bb_stack::{History, Hop, Place};

use crate::ExplorerView;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct GoBack;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct GoForward;

pub(crate) fn go_back(
    view: &mut ExplorerView,
    _: &GoBack,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    go(view, true, window, cx);
}

pub(crate) fn go_forward(
    view: &mut ExplorerView,
    _: &GoForward,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    go(view, false, window, cx);
}

/// Back (`back`) or Forward, from the keys or the title bar's arrows. With
/// no tab open it still goes; a closed file reopens at its place; a press
/// that skips or cannot go says why in the status bar (BB-BUG-080).
pub(crate) fn go(
    view: &mut ExplorerView,
    back: bool,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let here: Option<Place> = view.tabs.here();
    let hop: Hop = view.tabs.hop(back, here);
    if let Some(text) = said(back, &hop) {
        view.report(&text);
    }
    if let Some(place) = hop.to {
        crate::intel::glue::goto_point(view, &place.path, place.at, window, cx);
    }
    cx.notify();
}

/// `path` failed to open: say which file and why, and drop every jump that
/// waited for it, so a later open is not moved by a dead target (BB-BUG-081).
pub(crate) fn open_failed(view: &mut ExplorerView, path: &Path, error: &dyn std::fmt::Display) {
    let _dead: Option<crate::intel::model::Jump> = view
        .intel
        .jump
        .take_if(|jump: &mut crate::intel::model::Jump| jump.path == path);
    let _dead_line: Option<(std::path::PathBuf, usize)> = view
        .tests
        .pending_jump
        .take_if(|(waiting, _): &mut (std::path::PathBuf, usize)| waiting == path);
    view.report(&format!("Cannot open {}: {error}", name_of(path)));
}

fn name_of(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name: &std::ffi::OsStr| name.to_string_lossy().into_owned(),
    )
}

/// The status line for a press: none for a plain hop.
fn said(back: bool, hop: &Hop) -> Option<String> {
    let name: &str = if back { "Back" } else { "Forward" };
    let file: Option<String> = hop
        .gone
        .first()
        .map(|path: &std::path::PathBuf| name_of(path));
    match (&hop.to, file) {
        (Some(_), None) => None,
        (Some(_), Some(file)) => Some(format!("{name} skipped {file}: it no longer exists")),
        (None, Some(file)) => Some(format!(
            "{name}: {file} no longer exists; no other place to go"
        )),
        (None, None) => Some(format!("{name}: no place to go")),
    }
}

impl super::super::Tabs {
    /// The active tab's file and cursor, for Back / Forward.
    pub(crate) fn here(&self) -> Option<Place> {
        let tab: &super::super::Tab = self.active_tab()?;
        let at: bb_text::LspPoint = tab.document.lsp_point(tab.document.cursor()).ok()?;
        Some(Place {
            path: tab.path.clone(),
            at,
        })
    }

    /// A jump is about to leave the current place: remember it.
    pub(crate) fn leave_here(&mut self) {
        if let Some(here) = self.here() {
            self.history.leave(here);
        }
    }

    /// Is `path` open in a tab?
    pub(crate) fn is_open(&self, path: &Path) -> bool {
        self.items()
            .iter()
            .any(|tab: &super::super::Tab| tab.path == path)
    }

    /// The title bar's ← (`back`) or → has a place it can reach.
    pub(crate) fn can_go(&self, back: bool) -> bool {
        let open: &dyn Fn(&Path) -> bool = &|path: &Path| self.is_open(path);
        if back {
            self.history.can_back(open)
        } else {
            self.history.can_forward(open)
        }
    }

    /// One Back (`back`) or Forward press from `here`.
    pub(crate) fn hop(&mut self, back: bool, here: Option<Place>) -> Hop {
        let items: &[super::super::Tab] = &self.items;
        let open: &dyn Fn(&Path) -> bool =
            &|path: &Path| items.iter().any(|tab: &super::super::Tab| tab.path == path);
        if back {
            self.history.back(here, open)
        } else {
            self.history.forward(here, open)
        }
    }

    pub(crate) fn new() -> Self {
        Self::default()
    }
}
