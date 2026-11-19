//! Which server a file's requests go to (BBEOD27/28): the one place that
//! decides, so a Rust or Python file never reaches Roslyn and a C# file
//! never reaches another server. C# files go to Roslyn, Rust files to
//! rust-analyzer, Python files to pyright; a file of no language (a README,
//! `Cargo.toml`) goes to none (BB-BUG-090: it went to Roslyn).

use std::path::{Path, PathBuf};

use bb_intel::Server;
use bb_lang::Lang;
use gpui::{Context, Window};
use tokio::sync::mpsc::UnboundedReceiver;

use bb_text::CharEdit;

use crate::editor::Tab;
use crate::intel::bb_worker::protocol::{Event, Request};
use crate::intel::bb_worker::{self, Link};
use crate::intel::model::{LoadStatus, Sent, ServerStatus};
use crate::ExplorerView;

/// What the status bar says for a question about a file of no language.
pub(crate) const NO_SERVER: &str = "No language server for this file";

/// The server of each language: one exhaustive match, so a new language
/// gets a server or fails to compile, never falls through to Roslyn.
pub(crate) const fn server_of(lang: Lang) -> Server {
    match lang {
        Lang::CSharp => Server::Roslyn,
        Lang::Rust => Server::RustAnalyzer,
        Lang::Python => Server::Pyright,
    }
}

/// The server that serves `path`; `None` for a file of no language.
pub(crate) fn server_for(path: &Path) -> Option<Server> {
    Lang::of_source(path).map(server_of)
}

/// The link and status of a server next to Roslyn; `None` for Roslyn.
pub(crate) fn side(
    view: &mut ExplorerView,
    server: Server,
) -> Option<(&mut Option<Link>, &mut ServerStatus)> {
    match server {
        Server::Roslyn => None,
        Server::RustAnalyzer => Some((&mut view.rust.link, &mut view.rust.status)),
        Server::Pyright => Some((&mut view.python.link, &mut view.python.status)),
    }
}

/// The status of a server next to Roslyn (`Off` for Roslyn).
fn side_status(view: &ExplorerView, server: Server) -> (bool, &ServerStatus) {
    match server {
        Server::Roslyn => (false, &ServerStatus::Off),
        Server::RustAnalyzer => (view.rust.link.is_some(), &view.rust.status),
        Server::Pyright => (view.python.link.is_some(), &view.python.status),
    }
}

/// Whether `path`'s server is connected and ready for questions.
pub(crate) fn ready_for(view: &ExplorerView, path: &Path) -> bool {
    let Some(server): Option<Server> = server_for(path) else {
        return false;
    };
    match server {
        Server::Roslyn => view.link.is_some() && view.intel.status == LoadStatus::Ready,
        other @ (Server::RustAnalyzer | Server::Pyright) => {
            let (linked, status): (bool, &ServerStatus) = side_status(view, other);
            linked && *status == ServerStatus::Ready
        }
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
    let Some(server): Option<Server> = server_for(path) else {
        return String::from(NO_SERVER);
    };
    match server {
        Server::Roslyn => format!("C# server: {}", view.intel.status_label()),
        other @ (Server::RustAnalyzer | Server::Pyright) => format!(
            "{}: {}",
            other.name(),
            side_status(view, other).1.label(other)
        ),
    }
}

/// Send `request` to the server of its file; a request whose server is
/// not running is dropped here.
pub(crate) fn send(view: &mut ExplorerView, request: Request) {
    let Some(server): Option<Server> = request.path().map_or(Some(Server::Roslyn), server_for)
    else {
        return;
    };
    let link: Option<&Link> = match server {
        Server::Roslyn => view.link.as_ref(),
        Server::RustAnalyzer => view.rust.link.as_ref(),
        Server::Pyright => view.python.link.as_ref(),
    };
    let Some(link): Option<&Link> = link else {
        return;
    };
    let sent: Option<Sent> = Sent::of(&request);
    if link.send(request) {
        view.intel.sent(sent);
        return;
    }
    if let Some((link, status)) = side(view, server) {
        *link = None;
        *status = ServerStatus::Failed(String::from("server thread stopped"));
    } else {
        view.link = None;
        view.intel
            .server_gone(LoadStatus::Failed(String::from("server thread stopped")));
    }
}

/// Start rust-analyzer for the workspace while Rust is active and it holds
/// a `Cargo.toml` (a window test spawns only through its test starter).
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
    view.rust.status = ServerStatus::Starting;
    let Some((link, events)): Option<(Link, UnboundedReceiver<Event>)> =
        bb_worker::spawn(Server::RustAnalyzer, &root)
    else {
        return;
    };
    super::attach_server(view, (Server::RustAnalyzer, link), events, window, cx);
}

/// Start pyright for the window root while Python is active (a window
/// test spawns only through its test starter).
pub(crate) fn connect_python(
    view: &mut ExplorerView,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    if !view.langs.start(bb_lang::Tool::Pyright) {
        return;
    }
    view.python.status = ServerStatus::Starting;
    let Some((link, events)): Option<(Link, UnboundedReceiver<Event>)> =
        bb_worker::spawn(Server::Pyright, &view.root)
    else {
        return;
    };
    super::attach_server(view, (Server::Pyright, link), events, window, cx);
}

/// Apply a lifecycle event of a side server's thread; `false` when the
/// event is not one (diagnostics and answers are shared) or `server` is
/// Roslyn.
pub(crate) fn side_lifecycle(view: &mut ExplorerView, server: Server, event: &Event) -> bool {
    let status: ServerStatus = match event {
        Event::Stopped => ServerStatus::Stopped,
        Event::Ready { .. } => ServerStatus::Ready,
        Event::Failed(reason) => ServerStatus::Failed(reason.clone()),
        Event::Unavailable(hint) => ServerStatus::Missing(hint.clone()),
        Event::Progress { .. }
        | Event::Diagnostics { .. }
        | Event::Answer { .. }
        | Event::SyncFailed(_) => return false,
    };
    let label: String = status.label(server);
    let Some((link, shown)): Option<(&mut Option<Link>, &mut ServerStatus)> = side(view, server)
    else {
        return false;
    };
    if status != ServerStatus::Ready {
        *link = None;
    }
    eprintln!("{}_server_status={label}", server.language_id());
    *shown = status;
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
    super::flush(view);
    super::connect(view, window, cx);
    resync_open(view, &[Server::Roslyn]);
    cx.notify();
}

/// Send every tab of the servers in `started` to them (`didOpen` with the
/// current text). Only those: the other servers still hold their tabs
/// open, and a second open fails (BB-LSP-013). Flush before the servers
/// start, so no other tab's edit or close is lost here.
pub(crate) fn resync_open(view: &mut ExplorerView, started: &[Server]) {
    let opens: Vec<Request> = view
        .tabs
        .items_mut()
        .iter_mut()
        .filter(|tab: &&mut Tab| {
            server_for(&tab.path).is_some_and(|server: Server| started.contains(&server))
        })
        .map(|tab: &mut Tab| {
            // The text sent below already holds these edits.
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
