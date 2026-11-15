//! The terminal (BBEOD11, Ctrl+backtick): the user's shells in the solution
//! folder, under the editor. Minimal on purpose: text, a cursor, keys and
//! paste. Open Terminal Here (BB-BUG-052) starts it in a folder, or moves
//! the active shell there with `cd`. Split (BB-FEAT-079: Cmd+\ in the
//! terminal, Ctrl+Shift+backtick, or `+`) adds a shell beside the others,
//! as VS Code does; each pane's ✕ or `exit` ends only that shell.

mod bb_session;
mod panes;
mod view;

use std::path::Path;

use gpui::{Context, FocusHandle, InteractiveElement, Window};

use crate::ExplorerView;
pub(crate) use bb_session::{Cell, Session};
pub(crate) use panes::{on_split, Pane, SplitTerminal};
pub(crate) use view::panel;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct ToggleTerminal;

pub(crate) struct Terminal {
    /// Left to right; each one shell.
    pub(crate) panes: Vec<Pane>,
    /// The pane that takes keys, Open Terminal Here and the next split.
    pub(crate) active: usize,
    pub(crate) open: bool,
    /// The number the next pane shows ("zsh 3"); never reused.
    pub(crate) next: u32,
}

impl Terminal {
    pub(crate) fn new() -> Self {
        Self {
            panes: Vec::new(),
            active: 0,
            open: false,
            next: 1,
        }
    }

    /// The active pane's shell, if any.
    pub(crate) fn session(&self) -> Option<&Session> {
        self.panes.get(self.active).map(|pane: &Pane| &pane.session)
    }

    fn focus(&self) -> Option<&FocusHandle> {
        self.panes.get(self.active).map(|pane: &Pane| &pane.focus)
    }
}

/// The terminal's window actions: show/hide and split.
pub(crate) fn actions(root: gpui::Div, cx: &mut Context<ExplorerView>) -> gpui::Div {
    root.on_action(cx.listener(on_toggle))
        .on_action(cx.listener(on_split))
}

/// Hide the panel and give the editor the keys; the shells keep running.
pub(crate) fn hide(view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>) {
    view.terminal.open = false;
    window.focus(&view.editor_focus, cx);
}

/// Ctrl+backtick: open (starting a shell once) and focus, or hide.
pub(crate) fn on_toggle(
    view: &mut ExplorerView,
    _: &ToggleTerminal,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    if view.terminal.open {
        hide(view, window, cx);
        cx.notify();
        return;
    }
    let root: std::path::PathBuf = view.root.clone();
    if !view.terminal.panes.is_empty() || panes::start(view, &root, window, cx) {
        show(view, window, cx);
    }
}

/// Open Terminal Here: the shell in `folder`. The active shell gets a
/// cleared line and `cd`.
pub(crate) fn open_in(
    view: &mut ExplorerView,
    folder: &Path,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let started: bool = match view.terminal.session() {
        Some(session) => {
            session.input(cd_line(folder));
            true
        }
        None => panes::start(view, folder, window, cx),
    };
    if started {
        show(view, window, cx);
    }
}

/// Ctrl+U (clear the typed line), then `cd -- 'folder'`, safely quoted.
pub(crate) fn cd_line(folder: &Path) -> Vec<u8> {
    let quoted: String = folder.to_string_lossy().replace('\'', "'\\''");
    format!("\u{15}cd -- '{quoted}'\r").into_bytes()
}

/// Show the panel with the active pane focused.
fn show(view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>) {
    view.terminal.open = true;
    if let Some(focus) = view.terminal.focus() {
        window.focus(focus, cx);
    }
    cx.notify();
}

#[cfg(test)]
mod tests;
