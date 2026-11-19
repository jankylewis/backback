//! BBEOD39's recording seam: each REAL server thread the window starts sits
//! behind two relay threads that log what crosses its link (every
//! `didOpen` with its server and text) and what comes back (the files a
//! server sent diagnostics for, and sync failures such as a second open).

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use bb_intel::Server;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use super::super::super::lang_tests::tools::live::bb_kit::Bridge;
use crate::intel::bb_worker::protocol::{Event, Request};
use crate::intel::bb_worker::{self, Link, Starter};

/// What crossed the links so far.
#[derive(Default)]
pub(super) struct Log {
    /// Each `didOpen`: the server start it went to (0 first), the server,
    /// its file and its text.
    pub(super) opens: Vec<(usize, Server, PathBuf, String)>,
    /// Each start's real link, in start order; taking one ends that thread.
    pub(super) links: Vec<Arc<Mutex<Option<Link>>>>,
    /// Each file a server sent diagnostics for.
    pub(super) diagnosed: Vec<PathBuf>,
    /// Each sync failure a server reported.
    pub(super) sync_failed: Vec<String>,
    /// Starts whose server thread ended (its events closed after shutdown).
    pub(super) ended: usize,
}

pub(super) type Shared = Arc<Mutex<Log>>;

/// The window starts its servers through `starter` while this lives.
pub(super) struct Starts;

impl Starts {
    pub(super) fn install(starter: Starter) -> Self {
        bb_worker::test_starter(Some(starter));
        Self
    }
}

impl Drop for Starts {
    fn drop(&mut self) {
        bb_worker::test_starter(None);
    }
}

/// Start each server for real behind the relays; its bridge joins
/// `bridges` for the test thread to pump.
pub(super) fn starter(log: &Shared, bridges: &Rc<RefCell<Vec<Bridge>>>) -> Starter {
    let (log, bridges): (Shared, Rc<RefCell<Vec<Bridge>>>) = (Arc::clone(log), Rc::clone(bridges));
    Box::new(move |server: Server, at: &Path| {
        let (link, events): (Link, UnboundedReceiver<Event>) = match server {
            Server::Roslyn => bb_worker::start(at),
            Server::RustAnalyzer => bb_worker::start_rust(at),
            Server::Pyright => bb_worker::start_python(at),
        };
        let (tap, taps): (UnboundedSender<Request>, UnboundedReceiver<Request>) =
            unbounded_channel();
        let (back, from): (UnboundedSender<Event>, UnboundedReceiver<Event>) = unbounded_channel();
        let held: Arc<Mutex<Option<Link>>> = Arc::new(Mutex::new(Some(link)));
        let start: usize = {
            let mut shared: std::sync::MutexGuard<'_, Log> = log.lock().unwrap();
            shared.links.push(Arc::clone(&held));
            shared.links.len() - 1
        };
        relay_requests((start, server, taps), held, Arc::clone(&log));
        relay_events(events, back, Arc::clone(&log));
        let (to, window_end): (UnboundedSender<Event>, UnboundedReceiver<Event>) =
            unbounded_channel();
        bridges.borrow_mut().push((from, to));
        (Link::new(tap), window_end)
    })
}

fn relay_requests(
    (start, server, mut taps): (usize, Server, UnboundedReceiver<Request>),
    link: Arc<Mutex<Option<Link>>>,
    log: Shared,
) {
    std::thread::spawn(move || {
        while let Some(request) = taps.blocking_recv() {
            if let Request::Open { path, text } = &request {
                log.lock()
                    .unwrap()
                    .opens
                    .push((start, server, path.clone(), text.clone()));
            }
            let sent: Option<bool> = link
                .lock()
                .unwrap()
                .as_ref()
                .map(|real: &Link| real.send(request));
            if sent != Some(true) {
                break;
            }
        }
        // The window let go: drop the real link so the server shuts down
        // now, not when the process exits (nextest LEAK, BB-GAT-018).
        let _gone: Option<Link> = link.lock().unwrap().take();
    });
}

fn relay_events(mut events: UnboundedReceiver<Event>, back: UnboundedSender<Event>, log: Shared) {
    std::thread::spawn(move || {
        while let Some(event) = events.blocking_recv() {
            if let Event::Diagnostics { path, .. } = &event {
                log.lock().unwrap().diagnosed.push(path.clone());
            }
            if let Event::SyncFailed(reason) = &event {
                log.lock().unwrap().sync_failed.push(reason.clone());
            }
            // A closed window no longer listens; keep draining to the end.
            let _unheard: Result<(), _> = back.send(event);
        }
        log.lock().unwrap().ended += 1;
    });
}

/// Wait until every server thread started so far has shut down, so none
/// outlives the test (BB-GAT-018); a server that never stops fails it.
pub(super) fn wait_ended(log: &Shared) {
    let since: std::time::Instant = std::time::Instant::now();
    loop {
        let (ended, started): (usize, usize) = {
            let shared: std::sync::MutexGuard<'_, Log> = log.lock().unwrap();
            (shared.ended, shared.links.len())
        };
        if ended >= started {
            return;
        }
        assert!(
            since.elapsed() < std::time::Duration::from_secs(60),
            "{ended} of {started} server threads ended"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

/// What a fake server got from the window.
pub(super) type Requests = UnboundedReceiver<Request>;

/// Starts fake servers: links whose requests the test reads, no events.
pub(super) fn fake_starts() -> (Starts, Rc<RefCell<Vec<Requests>>>) {
    let sent: Rc<RefCell<Vec<Requests>>> = Rc::default();
    let held: Rc<RefCell<Vec<Requests>>> = Rc::clone(&sent);
    let starts: Starts = Starts::install(Box::new(move |_: Server, _: &Path| {
        let (tx, rx): (UnboundedSender<Request>, Requests) = unbounded_channel();
        held.borrow_mut().push(rx);
        let (_events, window_end): (UnboundedSender<Event>, UnboundedReceiver<Event>) =
            unbounded_channel();
        (Link::new(tx), window_end)
    }));
    (starts, sent)
}

/// How many `didOpen`s each fake server got so far, in start order.
pub(super) fn opens_per_start(sent: &Rc<RefCell<Vec<Requests>>>) -> Vec<usize> {
    sent.borrow_mut()
        .iter_mut()
        .map(|requests: &mut Requests| {
            std::iter::from_fn(|| requests.try_recv().ok())
                .filter(|r: &Request| matches!(r, Request::Open { .. }))
                .count()
        })
        .collect()
}
