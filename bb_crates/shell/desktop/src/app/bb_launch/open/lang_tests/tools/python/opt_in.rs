//! BBEOD41 (headmate ruling 2026-11-20): Python is opt-in. A C# and Python
//! folder opened with no saved choice ticks C# only, starts no pyright
//! process and saves nothing; the same folder with Python ticked starts the
//! real pyright, so the process probe sees one when it runs (a missing
//! pyright fails when `BACKBACK_REQUIRE_TOOLS`).

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use bb_intel::Server;
use bb_lang::fixtures::{Fixture, FIXTURES};
use bb_lang::{Lang, LangSet, Tool};
use gpui::{App, TestAppContext, WindowHandle};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use super::super::super::{close_all, open_with, opened};
use super::super::live::bb_kit::tools_required;
use super::pyright_servers;
use crate::intel::bb_worker::protocol::{Event, Request};
use crate::intel::bb_worker::{self, Link};
use crate::ExplorerView;

/// The servers the window asked for, and the real pyright threads ended.
#[derive(Default)]
pub(super) struct Asked {
    pub(super) servers: Rc<RefCell<Vec<Server>>>,
    pub(super) opens: Rc<RefCell<Vec<UnboundedReceiver<Request>>>>,
    ended: Arc<AtomicUsize>,
    real: Rc<RefCell<usize>>,
}

/// pyright starts for real; every other server is a fake whose requests
/// the test reads. Dropped: the window starts nothing again.
pub(super) struct Starts(pub(super) Asked);

impl Starts {
    pub(super) fn install() -> Self {
        let asked: Asked = Asked::default();
        #[allow(clippy::type_complexity)]
        let (servers, opens, ended, real): (
            Rc<RefCell<Vec<Server>>>,
            Rc<RefCell<Vec<UnboundedReceiver<Request>>>>,
            Arc<AtomicUsize>,
            Rc<RefCell<usize>>,
        ) = (
            Rc::clone(&asked.servers),
            Rc::clone(&asked.opens),
            Arc::clone(&asked.ended),
            Rc::clone(&asked.real),
        );
        bb_worker::test_starter(Some(Box::new(move |server: Server, at: &Path| {
            servers.borrow_mut().push(server);
            if server == Server::Pyright {
                *real.borrow_mut() += 1;
                return real_pyright(at, Arc::clone(&ended));
            }
            let (tx, rx): (UnboundedSender<Request>, UnboundedReceiver<Request>) =
                unbounded_channel();
            opens.borrow_mut().push(rx);
            let (_events, window_end): (UnboundedSender<Event>, UnboundedReceiver<Event>) =
                unbounded_channel();
            (Link::new(tx), window_end)
        })));
        Self(asked)
    }

    /// Wait until every real pyright thread ended (BB-GAT-018).
    fn wait_ended(&self) {
        let since: Instant = Instant::now();
        while self.0.ended.load(Ordering::SeqCst) < *self.0.real.borrow() {
            assert!(
                since.elapsed() < Duration::from_secs(60),
                "pyright never ended"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for Starts {
    fn drop(&mut self) {
        bb_worker::test_starter(None);
    }
}

fn real_pyright(at: &Path, ended: Arc<AtomicUsize>) -> (Link, UnboundedReceiver<Event>) {
    let (link, mut events): (Link, UnboundedReceiver<Event>) = bb_worker::start_python(at);
    std::thread::spawn(move || {
        while events.blocking_recv().is_some() {}
        ended.fetch_add(1, Ordering::SeqCst);
    });
    let (_quiet, window_end): (UnboundedSender<Event>, UnboundedReceiver<Event>) =
        unbounded_channel();
    (link, window_end)
}

/// A scratch `cs_py` fixture and an empty choice store for one test.
pub(super) struct Unpicked {
    _base: tempfile::TempDir,
    _store: tempfile::TempDir,
    pub(super) root: PathBuf,
}

impl Unpicked {
    pub(super) fn cs_py() -> Self {
        let store: tempfile::TempDir = tempfile::tempdir().unwrap();
        crate::launch::recent::test_store(Some(store.path().to_path_buf()));
        let base: tempfile::TempDir = tempfile::tempdir().unwrap();
        let fixture: &Fixture = FIXTURES
            .iter()
            .find(|f: &&Fixture| f.name == "cs_py")
            .unwrap();
        let root: PathBuf = fixture.write(&base.path().canonicalize().unwrap()).unwrap();
        Self {
            _base: base,
            _store: store,
            root,
        }
    }

    /// Open the folder as the command line or an untouched Open... does.
    pub(super) fn open(&self, cx: &mut TestAppContext) -> WindowHandle<ExplorerView> {
        let start: crate::args::Start = crate::args::start_for(self.root.clone(), false);
        assert_eq!(start.ticked, None);
        assert!(cx.update(|cx: &mut App| crate::launch::open_workspace(&start, cx)));
        cx.run_until_parked();
        opened(cx).pop().unwrap()
    }
}

impl Drop for Unpicked {
    fn drop(&mut self) {
        crate::launch::recent::test_store(None);
    }
}

fn no_pyright_for(settle: Duration) -> Vec<String> {
    let since: Instant = Instant::now();
    while since.elapsed() < settle {
        let found: Vec<String> = pyright_servers();
        if !found.is_empty() {
            return found;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Vec::new()
}

fn pyright_within(limit: Duration) -> Vec<String> {
    let since: Instant = Instant::now();
    loop {
        let found: Vec<String> = pyright_servers();
        if !found.is_empty() || since.elapsed() > limit {
            return found;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[gpui::test]
fn a_csharp_and_python_folder_with_no_saved_choice_starts_no_pyright_process(
    cx: &mut TestAppContext,
) {
    let folder: Unpicked = Unpicked::cs_py();
    let starts: Starts = Starts::install();
    let window: WindowHandle<ExplorerView> = folder.open(cx);
    assert_eq!(no_pyright_for(Duration::from_secs(3)), Vec::<String>::new());
    let (ticked, started): (LangSet, Vec<Tool>) = window
        .read_with(cx, |v: &ExplorerView, _: &App| {
            (v.langs.langs.ticked(), v.langs.started.clone())
        })
        .unwrap();
    assert_eq!(ticked, LangSet::of(&[Lang::CSharp]), "{started:?}");
    assert_eq!(started, vec![Tool::Roslyn]);
    assert_eq!(*starts.0.servers.borrow(), vec![Server::Roslyn]);
    assert_eq!(
        crate::launch::recent::langs_of(&folder.root),
        None,
        "nothing saved"
    );
    close_all(cx);
    let python: LangSet = LangSet::of(&[Lang::CSharp, Lang::Python]);
    let _picked: WindowHandle<ExplorerView> = open_with(cx, &folder.root, python);
    assert!(starts.0.servers.borrow().contains(&Server::Pyright));
    let control: Vec<String> = pyright_within(Duration::from_secs(60));
    let installed: bool =
        bb_python::pyright::locate(&bb_python::pyright::Inputs::from_env()).is_ok();
    assert!(
        installed || !tools_required(),
        "BACKBACK_REQUIRE_TOOLS is set but no pyright runs"
    );
    assert_eq!(
        !control.is_empty(),
        installed,
        "the probe sees a real pyright: {control:?}"
    );
    eprintln!("bbeod41_control_pyright={control:?}");
    close_all(cx);
    starts.wait_ended();
    assert_eq!(
        pyright_servers(),
        Vec::<String>::new(),
        "pyright ended with its window"
    );
}
