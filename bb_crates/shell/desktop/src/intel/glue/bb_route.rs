//! Which server a file's requests go to (BBEOD27): the one place that
//! decides, so a Rust file never reaches Roslyn and a C# file never reaches
//! rust-analyzer. C# files (and files of no language, as before the picker)
//! go to Roslyn, Rust files to rust-analyzer, Python files to none yet.

use std::path::{Path, PathBuf};

use bb_intel::Server;
use bb_lang::Lang;
use gpui::{Context, Window};
use tokio::sync::mpsc::UnboundedReceiver;

use bb_text::CharEdit;

use crate::editor::{Tab, TabEvent};
use crate::intel::bb_worker::protocol::{Event, Request};
use crate::intel::bb_worker::{self, Link};
use crate::intel::model::{LoadStatus, RustStatus, Sent};
use crate::ExplorerView;

/// The server that serves `path`; `None` for a language with no server.
pub(crate) fn server_for(path: &Path) -> Option<Server> {
    match Lang::of_source(path) {
        Some(Lang::CSharp) | None => Some(Server::Roslyn),
        Some(Lang::Rust) => Some(Server::RustAnalyzer),
        Some(Lang::Python) => None,
    }
}

/// Whether `path`'s server is connected and ready for questions.
pub(crate) fn ready_for(view: &ExplorerView, path: &Path) -> bool {
    match server_for(path) {
        Some(Server::Roslyn) => view.link.is_some() && view.intel.status == LoadStatus::Ready,
        Some(Server::RustAnalyzer) => {
            view.rust.link.is_some() && view.rust.status == RustStatus::Ready
        }
        None => false,
    }
}

/// Whether the active tab's server is ready.
pub(crate) fn ready_active(view: &ExplorerView) -> bool {
    view.tabs
        .active_tab()
        .is_some_and(|tab: &crate::editor::Tab| ready_for(view, &tab.path))
}

/// What the status bar says when a question about `path` cannot be asked.
pub(crate) fn status_for(view: &ExplorerView, path: &Path) -> String {
    match server_for(path) {
        Some(Server::Roslyn) => format!("C# server: {}", view.intel.status_label()),
        Some(Server::RustAnalyzer) => format!("Rust server: {}", view.rust.status.label()),
        None => String::from("No language server for this file"),
    }
}

/// Send `request` to the server of its file; a request about a file with
/// no server (or no server running) is dropped here.
pub(crate) fn send(view: &mut ExplorerView, request: Request) {
    let server: Option<Server> = request.path().map_or(Some(Server::Roslyn), server_for);
    let link: Option<&Link> = match server {
        Some(Server::Roslyn) => view.link.as_ref(),
        Some(Server::RustAnalyzer) => view.rust.link.as_ref(),
        None => None,
    };
    let Some(link): Option<&Link> = link else {
        return;
    };
    let sent: Option<Sent> = Sent::of(&request);
    if link.send(request) {
        view.intel.sent(sent);
        return;
    }
    match server {
        Some(Server::RustAnalyzer) => {
            view.rust.link = None;
            view.rust.status = RustStatus::Failed(String::from("server thread stopped"));
        }
        Some(Server::Roslyn) | None => {
            view.link = None;
            view.intel
                .server_gone(LoadStatus::Failed(String::from("server thread stopped")));
        }
    }
}

/// Start rust-analyzer for the workspace while Rust is active and it holds
/// a `Cargo.toml` (window tests record the start, spawn nothing).
pub(crate) fn connect_rust(
    view: &mut ExplorerView,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let root: Option<PathBuf> = bb_worker::find_cargo(&view.root)
        .filter(|_: &PathBuf| view.langs.start(bb_lang::Tool::RustAnalyzer));
    let Some(root): Option<PathBuf> = root else {
        return;
    };
    view.rust.status = RustStatus::Starting;
    if cfg!(test) {
        return;
    }
    let (link, events): (Link, UnboundedReceiver<Event>) = bb_worker::start_rust(&root);
    super::attach_server(view, (Server::RustAnalyzer, link), events, window, cx);
}

/// Apply a lifecycle event of rust-analyzer's thread; `false` when the
/// event is not one (diagnostics and answers are shared).
pub(crate) fn rust_lifecycle(view: &mut ExplorerView, event: &Event) -> bool {
    let status: RustStatus = match event {
        Event::Stopped => RustStatus::Stopped,
        Event::Ready { .. } => RustStatus::Ready,
        Event::Failed(reason) => RustStatus::Failed(reason.clone()),
        Event::Unavailable(hint) => RustStatus::Missing(hint.clone()),
        Event::Progress { .. }
        | Event::Diagnostics { .. }
        | Event::Answer { .. }
        | Event::SyncFailed(_) => return false,
    };
    if status != RustStatus::Ready {
        view.rust.link = None;
    }
    eprintln!("rust_server_status={}", status.label());
    view.rust.status = status;
    true
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
    super::connect(view, window, cx);
    resync_open(view);
    cx.notify();
}

pub(crate) fn resync_open(view: &mut ExplorerView) {
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
