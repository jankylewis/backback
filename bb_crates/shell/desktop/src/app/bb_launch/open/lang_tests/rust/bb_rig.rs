//! The rig for the BBEOD27 window tests: a window on the Rust fixture with
//! two fake server threads, C# (Roslyn) and Rust (rust-analyzer).

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

pub(super) const CS: &str = "Tool/Tool.cs";
pub(super) const PY: &str = "tools/run.py";
pub(super) const LIB: &str = "alpha/src/lib.rs";
pub(super) const MATH: &str = "alpha/src/math.rs";

pub(super) struct RustRig {
    _dir: tempfile::TempDir,
    pub(super) root: PathBuf,
    pub(super) view: Entity<ExplorerView>,
    pub(super) csharp: UnboundedReceiver<Request>,
    pub(super) rust: UnboundedReceiver<Request>,
    pub(super) rust_events: UnboundedSender<Event>,
}

pub(super) type Fake = (
    (Link, UnboundedReceiver<Request>),
    (UnboundedSender<Event>, UnboundedReceiver<Event>),
);

pub(super) fn channel_pair() -> Fake {
    let (request_tx, requests): (UnboundedSender<Request>, UnboundedReceiver<Request>) =
        unbounded_channel();
    ((Link::new(request_tx), requests), unbounded_channel())
}

pub(super) fn ready() -> Event {
    Event::Ready {
        took: Duration::ZERO,
        failed: Vec::new(),
    }
}

/// A window on the Rust fixture plus a C# and a Python file, `ticked`
/// languages active, both fake servers attached and ready.
pub(super) fn rust_rig(
    cx: &mut TestAppContext,
    ticked: LangSet,
) -> (RustRig, &mut VisualTestContext) {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = dir.path().canonicalize().unwrap().join("ws");
    fixtures::write(&root, Shape::Clean).unwrap();
    for (file, text) in [(CS, "class Tool { }\n"), (PY, "def run():\n    return 1\n")] {
        std::fs::create_dir_all(root.join(file).parent().unwrap()).unwrap();
        std::fs::write(root.join(file), text).unwrap();
    }
    let tree: bb_explorer::Explorer = bb_explorer::Explorer::open(&root).unwrap();
    let workspace: (PathBuf, bool) = (root.clone(), false);
    let (view, vcx): (Entity<ExplorerView>, &mut VisualTestContext) =
        cx.add_window_view(|_: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
            crate::launch::fresh_view(tree, workspace, WindowAppearance::Dark, cx)
        });
    let ((cs_link, csharp), (cs_events, cs_rx)): Fake = channel_pair();
    let ((rs_link, rust), (rust_events, rs_rx)): Fake = channel_pair();
    let all: LangSet = LangSet::of(&Lang::ALL);
    view.update_in(
        vcx,
        |v: &mut ExplorerView, window: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
            v.langs = WindowLangs::new(Langs::new(ticked, all));
            crate::intel::glue::attach(v, cs_link, cs_rx, window, cx);
            crate::intel::glue::attach_server(
                v,
                (Server::RustAnalyzer, rs_link),
                rs_rx,
                window,
                cx,
            );
        },
    );
    cs_events.send(ready()).unwrap();
    rust_events.send(ready()).unwrap();
    vcx.run_until_parked();
    let rig: RustRig = RustRig {
        _dir: dir,
        root,
        view,
        csharp,
        rust,
        rust_events,
    };
    (rig, vcx)
}

pub(super) fn drain(requests: &mut UnboundedReceiver<Request>) -> Vec<Request> {
    let mut out: Vec<Request> = Vec::new();
    while let Ok(request) = requests.try_recv() {
        out.push(request);
    }
    out
}

pub(super) fn files(requests: &[Request]) -> Vec<(String, PathBuf)> {
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
    pub(super) fn open(&self, vcx: &mut VisualTestContext, file: &str) {
        let path: PathBuf = self.root.join(file);
        self.view.update_in(
            vcx,
            |v: &mut ExplorerView, window: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
                v.open_file(&path, window, cx);
            },
        );
        vcx.run_until_parked();
    }

    pub(super) fn cursor_after(&self, vcx: &mut VisualTestContext, needle: &str) {
        self.view.update(
            vcx,
            |v: &mut ExplorerView, _: &mut gpui::Context<ExplorerView>| {
                let doc: &mut bb_text::Document = v.tabs.active_mut().unwrap();
                let at: usize = doc.text().find(needle).unwrap() + needle.len();
                doc.set_cursor(bb_text::CharIdx::new(at)).unwrap();
            },
        );
    }

    pub(super) fn last_rust_ask(&mut self) -> (Ticket, Ask) {
        drain(&mut self.rust)
            .into_iter()
            .rev()
            .find_map(|r: Request| match r {
                Request::Ask { ticket, ask, .. } => Some((ticket, ask)),
                Request::KillForTest
                | Request::Open { .. }
                | Request::Change { .. }
                | Request::Close { .. } => None,
            })
            .expect("rust-analyzer was asked")
    }

    pub(super) fn reply(&self, vcx: &mut VisualTestContext, ticket: Ticket, answer: Answer) {
        let now: Instant = Instant::now();
        let stages: Stages = Stages {
            sent: now,
            started: now,
            answered: now,
        };
        self.rust_events
            .send(Event::Answer {
                ticket,
                answer,
                stages,
            })
            .unwrap();
        vcx.run_until_parked();
    }

    pub(super) fn active(&self, vcx: &mut VisualTestContext) -> (PathBuf, usize) {
        self.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
            let tab: &crate::editor::Tab = v.tabs.active_tab().unwrap();
            (tab.path.clone(), tab.document.cursor_line())
        })
    }
}

pub(super) fn kinds(sent: &[(String, PathBuf)], path: &PathBuf) -> Vec<String> {
    sent.iter()
        .filter(|(_, p): &&(String, PathBuf)| p == path)
        .map(|(k, _): &(String, PathBuf)| k.clone())
        .collect()
}
