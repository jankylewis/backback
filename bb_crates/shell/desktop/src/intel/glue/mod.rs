//! Between the window and the server thread. [`flush`] is the one place
//! buffer changes reach the server; [`on_event`] the one place answers
//! reach the window.

mod answers;
mod asks;
pub(crate) mod bb_keys;
#[cfg(test)]
mod window_tests;

use std::path::{Path, PathBuf};

use bb_text::{CharEdit, CharIdx, Document};
use gpui::{AsyncWindowContext, Context, WeakEntity, Window};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::editor::{Tab, TabEvent};
use crate::intel::bb_worker::protocol::{Event, Request};
use crate::intel::bb_worker::{self, Link};
use crate::intel::model::{LoadStatus, Now, Pending};
use crate::ExplorerView;

pub(crate) use answers::apply_rename;
pub(crate) use answers::{jump, opened};
pub(crate) use asks::{hover_move, word_start};
pub(crate) use bb_keys::RestartServer;

/// Start the server for the workspace's solution, if it has one, and pump
/// its events into the window.
pub(crate) fn connect(
    view: &mut ExplorerView,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let Some(solution): Option<PathBuf> = bb_worker::find_solution(&view.root) else {
        view.intel.status = LoadStatus::NoSolution;
        return;
    };
    let (link, events): (Link, UnboundedReceiver<Event>) = bb_worker::start(&solution);
    attach(view, link, events, window, cx);
}

/// Relaunch Roslyn and replay the current text of every open tab.
pub(crate) fn restart(
    view: &mut ExplorerView,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    if !matches!(
        view.intel.status,
        LoadStatus::Stopped | LoadStatus::Failed(_)
    ) {
        return;
    }
    view.intel.in_flight.clear();
    view.intel.dismiss();
    connect(view, window, cx);
    resync_open(view);
    cx.notify();
}

fn resync_open(view: &mut ExplorerView) {
    let _events: Vec<TabEvent> = view.tabs.take_events();
    let opens: Vec<Request> = view
        .tabs
        .items_mut()
        .iter_mut()
        .map(|tab: &mut Tab| {
            let _edits: Vec<CharEdit> = tab.document.take_char_edits();
            Request::Open {
                path: tab.path.clone(),
                text: tab.document.text(),
            }
        })
        .collect();
    for open in opens {
        send(view, open);
    }
}

/// Use `link` for requests and deliver `events` to the window in order.
pub(crate) fn attach(
    view: &mut ExplorerView,
    link: Link,
    mut events: UnboundedReceiver<Event>,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    view.link = Some(link);
    view.intel.status = LoadStatus::Loading {
        loaded: 0,
        total: 0,
    };
    cx.spawn_in(
        window,
        async move |view: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext| {
            while let Some(event) = events.recv().await {
                let delivered: gpui::Result<()> = view.update_in(
                    cx,
                    |view: &mut ExplorerView,
                     window: &mut Window,
                     cx: &mut Context<ExplorerView>| {
                        on_event(view, event, window, cx);
                    },
                );
                if delivered.is_err() {
                    break;
                }
            }
        },
    )
    .detach();
}

/// Send tab opens and closes, then every tab's new char edits, in order.
pub(crate) fn flush(view: &mut ExplorerView) {
    for event in view.tabs.take_events() {
        match event {
            TabEvent::Opened(path) => {
                let text: Option<String> = view.tabs.tab_mut(&path).map(|tab: &mut Tab| {
                    // The text sent below already holds these edits.
                    let _included: Vec<CharEdit> = tab.document.take_char_edits();
                    tab.document.text()
                });
                if let Some(text) = text {
                    send(view, Request::Open { path, text });
                }
            }
            TabEvent::Closed(path) => {
                view.intel.closed(&path);
                send(view, Request::Close { path });
            }
        }
    }
    let changes: Vec<Request> = view
        .tabs
        .items_mut()
        .iter_mut()
        .filter_map(|tab: &mut Tab| {
            let edits: Vec<CharEdit> = tab.document.take_char_edits();
            (!edits.is_empty()).then(|| Request::Change {
                path: tab.path.clone(),
                edits: edits.into_iter().map(to_intel).collect(),
            })
        })
        .collect();
    for change in changes {
        send(view, change);
    }
}

fn to_intel(edit: CharEdit) -> bb_intel::Edit {
    bb_intel::Edit {
        from: edit.from,
        to: edit.to,
        text: edit.text,
    }
}

fn send(view: &mut ExplorerView, request: Request) {
    let Some(link): Option<&Link> = view.link.as_ref() else {
        return;
    };
    if !link.send(request) {
        view.link = None;
        view.intel.status = LoadStatus::Failed(String::from("server thread stopped"));
    }
}

/// Apply one server event to the window.
pub(crate) fn on_event(
    view: &mut ExplorerView,
    event: Event,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    match event {
        Event::Stopped => {
            view.link = None;
            view.intel.server_stopped();
        }
        Event::Progress { loaded, total } => view.intel.status.progress(loaded, total),
        Event::Ready { took } => {
            view.intel.status = LoadStatus::Ready;
            eprintln!("intel_ready_ms={}", took.as_millis());
        }
        Event::Failed(reason) => view.intel.status = LoadStatus::Failed(reason),
        Event::Diagnostics { path, list } => {
            if view.tabs.tab_mut(&path).is_some() {
                view.intel.set_diagnostics(path, list);
            }
        }
        Event::Answer {
            ticket,
            answer,
            stages,
        } => {
            let Some(pending): Option<Pending> = take_pending(view, ticket) else {
                return;
            };
            answers::apply(view, &pending, answer, stages, window, cx);
        }
        Event::SyncFailed(reason) => view.report(&format!("C# server: {reason}")),
    }
    cx.notify();
}

fn take_pending(
    view: &mut ExplorerView,
    ticket: crate::intel::bb_worker::protocol::Ticket,
) -> Option<Pending> {
    let path: PathBuf = view.intel.in_flight.peek(ticket)?.path.clone();
    let now: Now = now_for(view, &path);
    view.intel.in_flight.accept(ticket, &now)
}

/// The editor as seen from a question about `path`.
pub(crate) fn now_for(view: &mut ExplorerView, path: &Path) -> Now {
    let revision: Option<u64> = view
        .tabs
        .tab_mut(path)
        .map(|tab: &mut Tab| tab.document.revision());
    let active: Option<&Tab> = view.tabs.active_tab();
    Now {
        active: active.map(|tab: &Tab| tab.path.clone()),
        revision,
        word_start: active.map(|tab: &Tab| word_start(&tab.document, tab.document.cursor())),
    }
}

/// The text from `from` to the cursor, if the cursor is not before it.
pub(crate) fn typed_since(document: &Document, from: CharIdx) -> Option<String> {
    let cursor: CharIdx = document.cursor();
    document
        .rope()
        .get_slice(from.raw()..cursor.raw())
        .map(|part: bb_text::RopeSlice<'_>| part.to_string())
}
