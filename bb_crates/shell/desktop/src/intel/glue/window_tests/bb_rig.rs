//! The rig for window tests: the real window in-process (GPUI test
//! platform) with a fake server thread. Keys go through
//! `dispatch_keystroke`, answers through the event channel, and every
//! assertion reads the window's own state.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use bb_intel::{CompletionEntry, DocPath, DocUri, FileChange, Location, Span, TextChange};
use bb_text::LspPoint;
use gpui::{Entity, TestAppContext, VisualTestContext, Window, WindowAppearance};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::intel::bb_worker::protocol::{Answer, Ask, Event, Place, Request, Stages, Ticket};
use crate::intel::bb_worker::Link;
use crate::ExplorerView;

pub(super) const A: &str = "class A { void M() { x } }\n";

pub(super) struct Rig {
    _dir: tempfile::TempDir,
    pub(super) root: PathBuf,
    pub(super) file: PathBuf,
    pub(super) view: Entity<ExplorerView>,
    pub(super) requests: UnboundedReceiver<Request>,
    pub(super) events: UnboundedSender<Event>,
}

/// A window with `A.cs` open and the fake server ready.
pub(super) fn rig(cx: &mut TestAppContext, no_write: bool) -> (Rig, &mut VisualTestContext) {
    rig_with(cx, no_write, true)
}

/// A window with `A.cs` open; the fake server is ready only if `ready`.
pub(super) fn rig_with(
    cx: &mut TestAppContext,
    no_write: bool,
    ready: bool,
) -> (Rig, &mut VisualTestContext) {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = dir.path().canonicalize().unwrap();
    let file: PathBuf = root.join("A.cs");
    std::fs::write(&file, A).unwrap();
    let tree: bb_explorer::Explorer = bb_explorer::Explorer::open(&root).unwrap();
    let workspace: (PathBuf, bool) = (root.clone(), no_write);
    let (view, vcx): (Entity<ExplorerView>, &mut VisualTestContext) =
        cx.add_window_view(|_: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
            crate::launch::fresh_view(tree, workspace, WindowAppearance::Dark, cx)
        });
    let (request_tx, requests): (UnboundedSender<Request>, UnboundedReceiver<Request>) =
        unbounded_channel();
    let (events, event_rx): (UnboundedSender<Event>, UnboundedReceiver<Event>) =
        unbounded_channel();
    let opened: PathBuf = file.clone();
    view.update_in(
        vcx,
        |v: &mut ExplorerView, window: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
            crate::intel::glue::attach(v, Link::new(request_tx), event_rx, window, cx);
            v.open_file(&opened, window, cx);
        },
    );
    if ready {
        events
            .send(Event::Ready {
                took: Duration::ZERO,
            })
            .unwrap();
    }
    vcx.run_until_parked();
    let mut rig: Rig = Rig {
        _dir: dir,
        root,
        file,
        view,
        requests,
        events,
    };
    let first: Vec<Request> = rig.drain();
    assert!(
        matches!(first.first(), Some(Request::Open { path, .. }) if *path == rig.file),
        "{first:?}"
    );
    (rig, vcx)
}

impl Rig {
    pub(super) fn drain(&mut self) -> Vec<Request> {
        let mut out: Vec<Request> = Vec::new();
        while let Ok(request) = self.requests.try_recv() {
            out.push(request);
        }
        out
    }

    pub(super) fn last_ask(&mut self) -> (Ticket, Ask) {
        self.drain()
            .into_iter()
            .rev()
            .find_map(|r: Request| match r {
                Request::Ask { ticket, ask, .. } => Some((ticket, ask)),
                Request::KillForTest
                | Request::Open { .. }
                | Request::Change { .. }
                | Request::Close { .. } => None,
            })
            .expect("a question was sent")
    }

    pub(super) fn reply(&self, vcx: &mut VisualTestContext, ticket: Ticket, answer: Answer) {
        let now: Instant = Instant::now();
        let stages: Stages = Stages {
            sent: now,
            started: now,
            answered: now,
        };
        self.events
            .send(Event::Answer {
                ticket,
                answer,
                stages,
            })
            .unwrap();
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

    pub(super) fn text(&self, vcx: &mut VisualTestContext) -> String {
        self.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
            v.tabs.active().unwrap().text()
        })
    }
}

pub(super) fn items(labels: &[&str]) -> Answer {
    Answer::Completion(
        labels
            .iter()
            .map(|l: &&str| CompletionEntry {
                label: (*l).to_owned(),
                kind: None,
                detail: None,
            })
            .collect(),
    )
}

pub(super) fn place(path: &Path, line: u32) -> Place {
    Place {
        location: Location {
            uri: DocUri::File(DocPath::new(path.to_path_buf()).unwrap()),
            span: Span {
                start: LspPoint { line, character: 0 },
                end: LspPoint { line, character: 1 },
            },
        },
        preview: String::from("line"),
    }
}

fn total_at(line: u32, from: u32, to: u32) -> TextChange {
    TextChange {
        span: Span {
            start: LspPoint {
                line,
                character: from,
            },
            end: LspPoint {
                line,
                character: to,
            },
        },
        new_text: String::from("Total"),
    }
}

pub(super) fn rename_plan(open: &Path, closed: &Path) -> Answer {
    Answer::Rename(vec![
        FileChange::Edit {
            uri: DocUri::File(DocPath::new(open.to_path_buf()).unwrap()),
            changes: vec![total_at(0, 21, 22)],
        },
        FileChange::Edit {
            uri: DocUri::File(DocPath::new(closed.to_path_buf()).unwrap()),
            changes: vec![total_at(0, 4, 5), total_at(0, 8, 9)],
        },
    ])
}

pub(super) fn ask_rename(rig: &mut Rig, vcx: &mut VisualTestContext) -> Ticket {
    rig.cursor_after(vcx, "M() { ");
    vcx.simulate_keystrokes("f2 backspace T o t a l enter");
    let (ticket, ask): (Ticket, Ask) = rig.last_ask();
    assert_eq!(
        ask,
        Ask::Rename(
            LspPoint {
                line: 0,
                character: 21
            },
            String::from("Total")
        )
    );
    ticket
}
