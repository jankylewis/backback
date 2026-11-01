//! The terminal (BBEOD11, Ctrl+backtick): the user's shell in the solution folder,
//! under the editor. Minimal on purpose: text, a cursor, keys and paste.

mod bb_session;
mod keys;
mod view;

use gpui::{AsyncWindowContext, Context, FocusHandle, WeakEntity, Window};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::ExplorerView;
pub(crate) use bb_session::Session;
pub(crate) use view::panel;

/// Panel height in px; the rows that fit are the terminal's lines.
pub(crate) const HEIGHT: f32 = 240.0;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct ToggleTerminal;

pub(crate) struct Terminal {
    pub(crate) session: Option<Session>,
    pub(crate) open: bool,
    pub(crate) focus: FocusHandle,
}

impl Terminal {
    pub(crate) fn new(focus: FocusHandle) -> Self {
        Self {
            session: None,
            open: false,
            focus,
        }
    }
}

/// Ctrl+backtick: open (starting the shell once) and focus, or hide.
pub(crate) fn on_toggle(
    view: &mut ExplorerView,
    _: &ToggleTerminal,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    if view.terminal.open {
        view.terminal.open = false;
        window.focus(&view.editor_focus, cx);
        cx.notify();
        return;
    }
    if view.terminal.session.is_none() {
        let (wake, woken): (UnboundedSender<()>, UnboundedReceiver<()>) = unbounded_channel();
        match Session::start(&view.root, 80, 12, wake) {
            Ok(session) => view.terminal.session = Some(session),
            Err(error) => {
                view.report(&format!("Terminal could not start: {error}"));
                cx.notify();
                return;
            }
        }
        // Seam: gpui's test scheduler refuses wake-ups from the PTY thread;
        // unit tests read the screen themselves.
        if !cfg!(test) {
            redraw_on_output(woken, window, cx);
        }
    }
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
