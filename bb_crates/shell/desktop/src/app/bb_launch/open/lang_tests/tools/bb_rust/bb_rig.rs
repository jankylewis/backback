//! The rig for the BBEOD27/28 window tests: a window on the Rust fixture
//! with fake server threads for C# (Roslyn) and Rust (rust-analyzer), and
//! for Python (pyright) on the Python fixture written beside it.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use bb_cargo::fixtures::{self, Shape};
use bb_intel::Server;
use bb_lang::{Lang, LangSet, Langs};
use gpui::{Entity, TestAppContext, VisualTestContext, Window, WindowAppearance};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::intel::bb_worker::protocol::{Answer, Ask, Event, Request, Stages, Ticket};
use crate::intel::bb_worker::Link;
use crate::launch::WindowLangs;
use crate::ExplorerView;

pub(crate) const CS: &str = "Tool/Tool.cs";
pub(crate) const PY: &str = "tools/run.py";
pub(crate) const LIB: &str = "alpha/src/lib.rs";
pub(crate) const MATH: &str = "alpha/src/math.rs";

pub(crate) struct RustRig {
    _dir: tempfile::TempDir,
    pub(crate) root: PathBuf,
    pub(crate) view: Entity<ExplorerView>,
    pub(crate) csharp: UnboundedReceiver<Request>,
    pub(crate) rust: UnboundedReceiver<Request>,
    pub(crate) rust_events: UnboundedSender<Event>,
    pub(crate) python: UnboundedReceiver<Request>,
    pub(crate) python_events: UnboundedSender<Event>,
}

pub(crate) type Fake = (
    (Link, UnboundedReceiver<Request>),
    (UnboundedSender<Event>, UnboundedReceiver<Event>),
);

pub(crate) fn channel_pair() -> Fake {
    let (request_tx, requests): (UnboundedSender<Request>, UnboundedReceiver<Request>) =
        unbounded_channel();
    ((Link::new(request_tx), requests), unbounded_channel())
}

pub(crate) fn ready() -> Event {
    Event::Ready {
        took: Duration::ZERO,
        failed: Vec::new(),
    }
}

/// A window on the Rust fixture plus a C# and a Python file, `ticked`
/// languages active, the C# and Rust fake servers attached and ready.
pub(crate) fn rust_rig(
    cx: &mut TestAppContext,
    ticked: LangSet,
) -> (RustRig, &mut VisualTestContext) {
    rig_with(cx, ticked, false)
}

/// The Rust fixture with a C# and a Python file, and with `python` the
/// Python fixture too, in a new temporary folder.
fn write_root(python: bool) -> (tempfile::TempDir, PathBuf) {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = dir.path().canonicalize().unwrap().join("ws");
    fixtures::write(&root, Shape::Clean).unwrap();
    for (file, text) in [(CS, "class Tool { }\n"), (PY, "def run():\n    return 1\n")] {
        std::fs::create_dir_all(root.join(file).parent().unwrap()).unwrap();
        std::fs::write(root.join(file), text).unwrap();
    }
    if python {
        bb_python::fixtures::write(&root, bb_python::fixtures::Shape::Clean).unwrap();
    }
    (dir, root)
}

/// As [`rust_rig`]; with `python`, the Python fixture is written into the
/// root too and a fake pyright is attached and ready (BBEOD28).
pub(crate) fn rig_with(
    cx: &mut TestAppContext,
    ticked: LangSet,
    python: bool,
) -> (RustRig, &mut VisualTestContext) {
    let (dir, root): (tempfile::TempDir, PathBuf) = write_root(python);
    let tree: bb_explorer::Explorer = bb_explorer::Explorer::open(&root).unwrap();
    let workspace: (PathBuf, bool) = (root.clone(), false);
    let (view, vcx): (Entity<ExplorerView>, &mut VisualTestContext) =
        cx.add_window_view(|_: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
            crate::launch::fresh_view(tree, workspace, WindowAppearance::Dark, cx)
        });
    let ((cs_link, csharp), (cs_events, cs_rx)): Fake = channel_pair();
    let ((rs_link, rust), (rust_events, rs_rx)): Fake = channel_pair();
    let ((py_link, python_rx), (python_events, py_rx)): Fake = channel_pair();
    let all: LangSet = LangSet::of(&Lang::ALL);
    view.update_in(
        vcx,
        |v: &mut ExplorerView, window: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
            v.langs = WindowLangs::new(Langs::new(ticked, all));
            crate::intel::glue::attach(v, cs_link, cs_rx, window, cx);
            let rust_analyzer: (Server, Link) = (Server::RustAnalyzer, rs_link);
            crate::intel::glue::attach_server(v, rust_analyzer, rs_rx, window, cx);
            if python {
                let pyright: (Server, Link) = (Server::Pyright, py_link);
                crate::intel::glue::attach_server(v, pyright, py_rx, window, cx);
            }
        },
    );
    let pyright: Option<&UnboundedSender<Event>> = python.then_some(&python_events);
    for events in [&cs_events, &rust_events].into_iter().chain(pyright) {
        events.send(ready()).unwrap();
    }
    vcx.run_until_parked();
    let rig: RustRig = RustRig {
        _dir: dir,
        root,
        view,
        csharp,
        rust,
        rust_events,
        python: python_rx,
        python_events,
    };
    (rig, vcx)
}

/// Answer `ticket` on the fake server thread whose events are `events`.
pub(crate) fn reply_on(
    events: &UnboundedSender<Event>,
    vcx: &mut VisualTestContext,
    (ticket, answer): (Ticket, Answer),
) {
    let now: Instant = Instant::now();
    let stages: Stages = Stages {
        sent: now,
        started: now,
        answered: now,
    };
    events
        .send(Event::Answer {
            ticket,
            answer,
            stages,
        })
        .unwrap();
    vcx.run_until_parked();
}

fn last_ask(requests: &mut UnboundedReceiver<Request>) -> (Ticket, Ask) {
    drain(requests)
        .into_iter()
        .rev()
        .find_map(|r: Request| match r {
            Request::Ask { ticket, ask, .. } => Some((ticket, ask)),
            Request::KillForTest
            | Request::Open { .. }
            | Request::Change { .. }
            | Request::Close { .. } => None,
        })
        .expect("the server was asked")
}

pub(crate) fn drain(requests: &mut UnboundedReceiver<Request>) -> Vec<Request> {
    let mut out: Vec<Request> = Vec::new();
    while let Ok(request) = requests.try_recv() {
        out.push(request);
    }
    out
}

pub(crate) fn files(requests: &[Request]) -> Vec<(String, PathBuf)> {
    requests
        .iter()
        .filter_map(|r: &Request| {
            let kind: &str = match r {
                Request::Open { .. } => "open",
                Request::Change { .. } => "change",
                Request::Ask { .. } => "ask",
                Request::Close { .. } | Request::KillForTest => "other",
            };
            r.path()
                .map(|p: &std::path::Path| (kind.to_owned(), p.to_path_buf()))
        })
        .collect()
}

impl RustRig {
    pub(crate) fn open(&self, vcx: &mut VisualTestContext, file: &str) {
        let path: PathBuf = self.root.join(file);
        self.view.update_in(
            vcx,
            |v: &mut ExplorerView, window: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
                v.open_file(&path, window, cx);
            },
        );
        vcx.run_until_parked();
    }

    pub(crate) fn cursor_after(&self, vcx: &mut VisualTestContext, needle: &str) {
        self.view.update(
            vcx,
            |v: &mut ExplorerView, _: &mut gpui::Context<ExplorerView>| {
                let doc: &mut bb_text::Document = v.tabs.active_mut().unwrap();
                let at: usize = doc.text().find(needle).unwrap() + needle.len();
                doc.set_cursor(bb_text::CharIdx::new(at)).unwrap();
            },
        );
    }

    pub(crate) fn last_rust_ask(&mut self) -> (Ticket, Ask) {
        last_ask(&mut self.rust)
    }

    pub(crate) fn last_python_ask(&mut self) -> (Ticket, Ask) {
        last_ask(&mut self.python)
    }

    pub(crate) fn reply(&self, vcx: &mut VisualTestContext, ticket: Ticket, answer: Answer) {
        reply_on(&self.rust_events, vcx, (ticket, answer));
    }

    pub(crate) fn active(&self, vcx: &mut VisualTestContext) -> (PathBuf, usize) {
        self.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
            let tab: &crate::editor::Tab = v.tabs.active_tab().unwrap();
            (tab.path.clone(), tab.document.cursor_line())
        })
    }
}

pub(crate) fn kinds(sent: &[(String, PathBuf)], path: &PathBuf) -> Vec<String> {
    sent.iter()
        .filter(|(_, p): &&(String, PathBuf)| p == path)
        .map(|(k, _): &(String, PathBuf)| k.clone())
        .collect()
}
