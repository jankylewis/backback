//! Between the window and the server thread: [`flush`] is the one place buffer
//! changes reach the server; [`on_event`] the one place answers reach the window.

mod answers;
pub(crate) mod input;
#[path = "bb_route.rs"]
pub(crate) mod route;
#[cfg(test)]
pub(crate) mod window_tests;

use std::path::{Path, PathBuf};

use bb_text::{CharEdit, CharIdx, Document};
use gpui::{AsyncWindowContext, Context, WeakEntity, Window};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::editor::{Tab, TabEvent};
use crate::intel::bb_worker::protocol::{Event, Request};
use crate::intel::bb_worker::{self, Link};
use crate::intel::model::{LoadStatus, Now, Pending};
use crate::ExplorerView;
use bb_intel::Server;
use route::send;

pub(crate) use answers::apply_rename;
pub(crate) use answers::{goto_point, jump, opened};
pub(crate) use input::asks::{ask, ask_at_cursor, hover_ask, hover_move, word_start};
pub(crate) use input::bb_keys;
pub(crate) use input::bb_keys::RestartServer;
pub(crate) use route::{connect_python, connect_rust, restart};

/// Start Roslyn for the workspace's solution while C# is active (BBEOD26;
/// window tests record the start, spawn nothing) and pump its events.
pub(crate) fn connect(
    view: &mut ExplorerView,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let started: Option<PathBuf> = bb_worker::find_solution(&view.root)
        .filter(|_: &PathBuf| view.langs.start(bb_lang::Tool::Roslyn) && !cfg!(test));
    let Some(solution): Option<PathBuf> = started else {
        view.intel.status = LoadStatus::NoSolution;
        return;
    };
    let (link, events): (Link, UnboundedReceiver<Event>) = bb_worker::start(&solution);
    attach(view, link, events, window, cx);
}

/// Use `link` for Roslyn's requests and deliver `events` to the window in order.
pub(crate) fn attach(
    view: &mut ExplorerView,
    link: Link,
    events: UnboundedReceiver<Event>,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    attach_server(view, (Server::Roslyn, link), events, window, cx);
}

/// Use `link` for `server`'s requests and deliver `events` to the window
/// in order, each tagged with the server it came from.
pub(crate) fn attach_server(
    view: &mut ExplorerView,
    (server, link): (Server, Link),
    mut events: UnboundedReceiver<Event>,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    match server {
        Server::Roslyn => {
            view.link = Some(link);
            view.intel.status = LoadStatus::Loading {
                loaded: 0,
                total: 0,
            };
        }
        Server::RustAnalyzer => view.rust.link = Some(link),
        Server::Pyright => view.python.link = Some(link),
    }
    cx.spawn_in(
        window,
        async move |view: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext| {
            while let Some(event) = events.recv().await {
                let delivered: gpui::Result<()> = view.update_in(
                    cx,
                    |view: &mut ExplorerView,
                     window: &mut Window,
                     cx: &mut Context<ExplorerView>| {
                        on_event(view, (server, event), window, cx);
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

/// Apply one event of `server`'s thread to the window.
pub(crate) fn on_event(
    view: &mut ExplorerView,
    (server, event): (Server, Event),
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    if route::side_lifecycle(view, server, &event) {
        cx.notify();
        return;
    }
    match event {
        Event::Stopped => {
            view.link = None;
            view.intel.server_gone(LoadStatus::Stopped);
        }
        Event::Progress { loaded, total } => view.intel.status.progress(loaded, total),
        Event::Ready { took, failed } => {
            eprintln!("intel_ready_ms={}", took.as_millis());
            view.intel.ready(failed);
        }
        Event::Failed(reason) | Event::Unavailable(reason) => {
            view.link = None;
            view.intel.server_gone(LoadStatus::Failed(reason));
        }
        Event::Diagnostics {
            path,
            version,
            list,
        } => {
            if view.tabs.tab_mut(&path).is_some() {
                view.intel.set_diagnostics(path, version, list);
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
        Event::SyncFailed(reason) => view.report(&format!("{}: {reason}", server.name())),
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
