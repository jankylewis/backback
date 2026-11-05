//! The terminal (BBEOD11, Ctrl+backtick): the user's shell in the solution folder,
//! under the editor. Minimal on purpose: text, a cursor, keys and paste.
//! Open Terminal Here (BB-BUG-052) starts it in a folder, or moves the
//! running shell there with `cd`.

mod bb_session;

mod view;

use std::path::Path;

use gpui::{AsyncWindowContext, Context, FocusHandle, WeakEntity, Window};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::ExplorerView;
pub(crate) use bb_session::{Cell, Session};
pub(crate) use view::panel;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct ToggleTerminal;

pub(crate) struct Terminal {
    pub(crate) session: Option<Session>,
    pub(crate) open: bool,
    pub(crate) focus: FocusHandle,
    /// The grid's place on screen last frame, for mouse cells.
    pub(crate) grid: gpui::ScrollHandle,
    /// A mouse press began a selection and the button is still down.
    pub(crate) selecting: bool,
}

impl Terminal {
    pub(crate) fn new(focus: FocusHandle) -> Self {
        Self {
            session: None,
            open: false,
            focus,
            grid: gpui::ScrollHandle::new(),
            selecting: false,
        }
    }
}

/// Hide the panel and give the editor the keys; the shell keeps running.
pub(crate) fn hide(view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>) {
    view.terminal.open = false;
    window.focus(&view.editor_focus, cx);
}

/// Ctrl+backtick: open (starting the shell once) and focus, or hide.
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
    if view.terminal.session.is_some() || start(view, &root, window, cx) {
        show(view, window, cx);
    }
}

/// Open Terminal Here: the shell in `folder`. A running shell gets a
/// cleared line and `cd` (the panel keeps one shell, as before).
pub(crate) fn open_in(
    view: &mut ExplorerView,
    folder: &Path,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let started: bool = match &view.terminal.session {
        Some(session) => {
            session.input(cd_line(folder));
            true
        }
        None => start(view, folder, window, cx),
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

/// Start the shell in `cwd`; false (and a status) when it cannot start.
fn start(
    view: &mut ExplorerView,
    cwd: &Path,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) -> bool {
    let (wake, woken): (UnboundedSender<()>, UnboundedReceiver<()>) = unbounded_channel();
    match Session::start(cwd, 80, 12, wake) {
        Ok(session) => view.terminal.session = Some(session),
        Err(error) => {
            view.report(&format!("Terminal could not start: {error}"));
            cx.notify();
            return false;
        }
    }
    // Seam: gpui's test scheduler refuses wake-ups from the PTY thread;
    // unit tests read the screen themselves.
    if !cfg!(test) {
        redraw_on_output(woken, window, cx);
    }
    true
}

fn show(view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>) {
    view.terminal.open = true;
    window.focus(&view.terminal.focus, cx);
    cx.notify();
}

/// One redraw per burst of output, however much the shell prints.
fn redraw_on_output(
    mut woken: UnboundedReceiver<()>,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    cx.spawn_in(
        window,
        async move |view: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext| {
            while woken.recv().await.is_some() {
                while woken.try_recv().is_ok() {}
                if view
                    .update(
                        cx,
                        |_: &mut ExplorerView, cx: &mut Context<ExplorerView>| cx.notify(),
                    )
                    .is_err()
                {
                    return;
                }
            }
        },
    )
    .detach();
}

#[cfg(test)]
mod tests;
