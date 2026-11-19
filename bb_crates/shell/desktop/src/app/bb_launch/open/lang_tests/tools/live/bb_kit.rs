//! BBEOD29: a window on the REAL rust-analyzer or pyright, driven by mouse
//! and keys. A server answers as it indexes, so each step acts again until
//! the window shows the result or [`STEP`] passes, then fails naming what
//! it waited for and what the window showed.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use gpui::{App, Modifiers, VisualTestContext, WindowHandle};

use super::super::super::watchdog::{watchdog, Watchdog};
use crate::intel::bb_worker::protocol::Event;
use crate::ExplorerView;

/// How long one step on a real server may take.
pub(crate) const STEP: Duration = Duration::from_secs(120);

type Events = tokio::sync::mpsc::UnboundedReceiver<Event>;
/// A real server's events and the window's channel: gpui panics when a foreign
/// thread wakes its tasks, so the test thread moves each across (`try_recv`).
pub(crate) type Bridge = (Events, tokio::sync::mpsc::UnboundedSender<Event>);

/// A window on a fixture and the fixture's folder.
pub(crate) struct Live {
    pub(crate) root: PathBuf,
    pub(crate) window: WindowHandle<ExplorerView>,
    pub(crate) vcx: VisualTestContext,
    /// One bridge per real server the window talks to.
    pub(crate) bridges: Rc<RefCell<Vec<Bridge>>>,
}

impl Live {
    pub(crate) fn new(
        root: PathBuf,
        window: WindowHandle<ExplorerView>,
        vcx: VisualTestContext,
    ) -> Self {
        Self {
            root,
            window,
            vcx,
            bridges: Rc::default(),
        }
    }

    /// Move the events the server thread sent so far into the window.
    fn pump(&mut self) {
        for (from, to) in self.bridges.borrow_mut().iter_mut() {
            while let Ok(event) = from.try_recv() {
                to.send(event).unwrap();
            }
        }
        self.vcx.run_until_parked();
    }
}

/// What the window shows, for a failure message.
fn seen(v: &ExplorerView) -> String {
    let tab: Option<(PathBuf, usize)> = v
        .tabs
        .active_tab()
        .map(|t: &crate::editor::Tab| (t.path.clone(), t.document.cursor_line()));
    format!(
        "status_line={:?} status={:?} tab={tab:?} problems={}",
        crate::launch::langs::status_line(v),
        v.status,
        v.intel.problems().len()
    )
}

impl Live {
    pub(crate) fn read<T>(&mut self, read: impl FnOnce(&ExplorerView) -> T) -> T {
        self.window
            .read_with(&self.vcx, |v: &ExplorerView, _: &App| read(v))
            .unwrap()
    }

    /// Run `act`, then wait for `done`; act again every 2 s until [`STEP`].
    pub(crate) fn until(
        &mut self,
        what: &str,
        act: &mut dyn FnMut(&mut Self),
        done: &dyn Fn(&ExplorerView) -> bool,
    ) {
        let _armed: Watchdog = watchdog(what, STEP + STEP);
        let started: Instant = Instant::now();
        let mut acted: Option<Instant> = None;
        loop {
            if acted.is_none_or(|at: Instant| at.elapsed() > Duration::from_secs(2)) {
                act(self);
                acted = Some(Instant::now());
            }
            self.pump();
            if self.read(done) {
                return;
            }
            let window: String = self.read(seen);
            assert!(
                started.elapsed() < STEP,
                "waited {STEP:?} for {what}; window: {window}"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Start the REAL server thread of `server` on the root and attach it
    /// as the app does (`connect_rust`/`connect_python` start nothing in a
    /// test build).
    pub(crate) fn attach(&mut self, server: bb_intel::Server) {
        let started: (crate::intel::bb_worker::Link, Events) = match server {
            bb_intel::Server::RustAnalyzer => crate::intel::bb_worker::start_rust(&self.root),
            bb_intel::Server::Pyright => crate::intel::bb_worker::start_python(&self.root),
            bb_intel::Server::Roslyn => {
                let solution: Option<PathBuf> = crate::intel::bb_worker::find_solution(&self.root);
                crate::intel::bb_worker::start(&solution.unwrap())
            }
        };
        let (to, window_end): (tokio::sync::mpsc::UnboundedSender<Event>, Events) =
            tokio::sync::mpsc::unbounded_channel();
        self.bridges.borrow_mut().push((started.1, to));
        let link: crate::intel::bb_worker::Link = started.0;
        self.window
            .update(
                &mut self.vcx,
                |v: &mut ExplorerView,
                 w: &mut gpui::Window,
                 cx: &mut gpui::Context<ExplorerView>| {
                    crate::intel::glue::attach_server(v, (server, link), window_end, w, cx);
                },
            )
            .unwrap();
    }

    pub(crate) fn open(&mut self, file: &str) {
        let path: PathBuf = self.root.join(file);
        self.window
            .update(
                &mut self.vcx,
                |v: &mut ExplorerView,
                 w: &mut gpui::Window,
                 cx: &mut gpui::Context<ExplorerView>| {
                    v.open_file(&path, w, cx);
                },
            )
            .unwrap();
        self.vcx.run_until_parked();
    }

    pub(crate) fn cursor_after(&mut self, needle: &str) {
        self.window
            .update(
                &mut self.vcx,
                |v: &mut ExplorerView,
                 _: &mut gpui::Window,
                 _: &mut gpui::Context<ExplorerView>| {
                    let doc: &mut bb_text::Document = v.tabs.active_mut().unwrap();
                    let at: usize = doc.text().find(needle).unwrap() + needle.len();
                    doc.set_cursor(bb_text::CharIdx::new(at)).unwrap();
                },
            )
            .unwrap();
    }

    /// The window point over column `column` of 0-based line `line`.
    pub(crate) fn point(&mut self, line: u32, column: u32) -> gpui::Point<gpui::Pixels> {
        let row: gpui::Bounds<gpui::Pixels> = self
            .vcx
            .debug_bounds(format!("editor-line-{line}").leak())
            .unwrap();
        let x: f32 =
            f32::from(row.origin.x) + 56.0 + 7.8 * f32::from(u16::try_from(column).unwrap()) + 1.0;
        gpui::point(gpui::px(x), row.center().y)
    }

    pub(crate) fn center_of(&mut self, selector: &'static str) -> gpui::Point<gpui::Pixels> {
        self.vcx.debug_bounds(selector).unwrap().center()
    }

    pub(crate) fn click_at(&mut self, at: gpui::Point<gpui::Pixels>, mods: Modifiers) {
        self.vcx.simulate_click(at, mods);
    }

    /// Rest the mouse at `at` past the hover delay.
    pub(crate) fn rest_mouse(&mut self, at: gpui::Point<gpui::Pixels>) {
        self.vcx.simulate_mouse_move(at, None, Modifiers::default());
        self.vcx
            .executor()
            .advance_clock(Duration::from_millis(400));
    }

    pub(crate) fn keys(&mut self, keys: &str) {
        self.vcx.simulate_keystrokes(keys);
    }

    /// The Problems row of the error on 0-based `line` of `file`.
    pub(crate) fn error_row(&mut self, file: &Path, line: u32) -> usize {
        self.read(|v: &ExplorerView| {
            v.intel
                .problems_in(&v.root)
                .iter()
                .position(|(p, d): &(&Path, &bb_intel::Diagnostic)| {
                    *p == file
                        && d.span.start.line == line
                        && d.severity == Some(bb_intel::Severity::Error)
                })
                .unwrap()
        })
    }

    pub(crate) fn tab_text(&mut self, path: &Path) -> Option<String> {
        self.read(|v: &ExplorerView| {
            v.tabs
                .items()
                .iter()
                .find(|t: &&crate::editor::Tab| t.path == path)
                .map(|t: &crate::editor::Tab| t.document.text())
        })
    }
}

/// The active tab's file and cursor line.
pub(crate) fn active(v: &ExplorerView) -> Option<(PathBuf, usize)> {
    v.tabs
        .active_tab()
        .map(|t: &crate::editor::Tab| (t.path.clone(), t.document.cursor_line()))
}

/// Every file and folder under `root`, sorted.
pub(crate) fn listing(root: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut todo: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = todo.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path: PathBuf = entry.unwrap().path();
            if path.is_dir() {
                todo.push(path.clone());
            }
            found.push(path);
        }
    }
    found.sort();
    found
}

/// True when `BACKBACK_REQUIRE_TOOLS` is set: a missing server fails.
pub(crate) fn tools_required() -> bool {
    std::env::var_os("BACKBACK_REQUIRE_TOOLS").is_some()
}
