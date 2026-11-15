//! The terminal's panes (BB-FEAT-079): one shell each, side by side. A
//! split starts a new shell in the solution folder beside the others; a
//! pane closes on its ✕ or when its shell exits, and the last one to close
//! hides the panel.

use std::path::Path;

use gpui::{AsyncWindowContext, Context, FocusHandle, WeakEntity, Window};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

mod bb_title;

use super::Session;
use crate::ExplorerView;
pub(crate) use bb_title::title;

/// Cmd+\ in the terminal, Ctrl+Shift+backtick anywhere: one more shell.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct SplitTerminal;

pub(crate) struct Pane {
    pub(crate) session: Session,
    pub(crate) focus: FocusHandle,
    /// Its shown number ("zsh 2").
    pub(crate) number: u32,
    /// The grid's place on screen last frame, for mouse cells.
    pub(crate) grid: gpui::ScrollHandle,
    /// A mouse press began a selection and the button is still down.
    pub(crate) selecting: bool,
    /// Wheel pixels not yet a whole line (a trackpad sends small steps).
    pub(crate) scroll_rest: f32,
}

/// Start a shell in `cwd` as a new pane right of the active one, and make
/// it active; false (and a status) when it cannot start.
pub(crate) fn start(
    view: &mut ExplorerView,
    cwd: &Path,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) -> bool {
    let (wake, woken): (UnboundedSender<()>, UnboundedReceiver<()>) = unbounded_channel();
    let session: Session = match Session::start(cwd, 80, 12, wake) {
        Ok(session) => session,
        Err(error) => {
            view.report(&format!("Terminal could not start: {error}"));
            cx.notify();
            return false;
        }
    };
    let terminal: &mut super::Terminal = &mut view.terminal;
    let at: usize = if terminal.panes.is_empty() {
        0
    } else {
        terminal.active.saturating_add(1).min(terminal.panes.len())
    };
    terminal.panes.insert(
        at,
        Pane {
            session,
            focus: cx.focus_handle(),
            number: terminal.next,
            grid: gpui::ScrollHandle::new(),
            selecting: false,
            scroll_rest: 0.0,
        },
    );
    terminal.next = terminal.next.saturating_add(1);
    terminal.active = at;
    // Seam: gpui's test scheduler refuses wake-ups from the PTY thread;
    // unit tests read the screen themselves.
    if !cfg!(test) {
        redraw_on_output(woken, window, cx);
    }
    true
}

/// Split: a new shell beside the active one, shown and focused.
pub(crate) fn on_split(
    view: &mut ExplorerView,
    _: &SplitTerminal,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let root: std::path::PathBuf = view.root.clone();
    if start(view, &root, window, cx) {
        super::show(view, window, cx);
    }
}

/// End the shell of pane `index`; the one left of it takes the keys. The
/// last pane gone hides the panel.
pub(crate) fn close(
    view: &mut ExplorerView,
    index: usize,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let terminal: &mut super::Terminal = &mut view.terminal;
    if index >= terminal.panes.len() {
        return;
    }
    drop(terminal.panes.remove(index));
    if terminal.active >= index && terminal.active > 0 {
        terminal.active -= 1;
    }
    if terminal.panes.is_empty() {
        super::hide(view, window, cx);
    } else if view.terminal.open {
        super::show(view, window, cx);
    }
    cx.notify();
}

/// Close every pane whose shell has exited (`exit`, Ctrl+D).
pub(crate) fn reap(view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>) {
    while let Some(index) = view
        .terminal
        .panes
        .iter()
        .position(|pane: &Pane| pane.session.exited())
    {
        close(view, index, window, cx);
    }
}

/// One redraw per burst of output, however much the shell prints; a shell
/// that exited takes its pane with it.
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
                let updated: Result<(), _> = view.update_in(
                    cx,
                    |view: &mut ExplorerView,
                     window: &mut Window,
                     cx: &mut Context<ExplorerView>| {
                        reap(view, window, cx);
                        cx.notify();
                    },
                );
                if updated.is_err() {
                    return;
                }
            }
        },
    )
    .detach();
}

#[cfg(test)]
mod tests;
