//! Waiting on the worker's events: the next one, the diagnostics after an
//! Open, an answer by ticket, and the whole load until Ready.

use std::path::Path;
use std::time::Duration;

use tokio::sync::mpsc::UnboundedReceiver;

use super::{LOAD, WAIT};
use crate::intel::bb_worker::protocol::{Answer, Ask, Event, Request, Stages, Ticket};
use crate::intel::bb_worker::Link;

pub(super) async fn next(events: &mut UnboundedReceiver<Event>) -> Event {
    tokio::time::timeout(WAIT, events.recv())
        .await
        .expect("an event before the deadline")
        .expect("the server thread is alive")
}

/// The first diagnostics after an Open. A late project-load batch may still
/// report `Progress` after Ready (BB-LSP-002); any other event fails, named.
pub(super) async fn first_diagnostics(events: &mut UnboundedReceiver<Event>) {
    loop {
        match next(events).await {
            Event::Diagnostics { .. } => return,
            Event::Progress { .. } => {}
            other @ (Event::Stopped
            | Event::Ready { .. }
            | Event::Failed(_)
            | Event::Answer { .. }
            | Event::SyncFailed(_)) => {
                panic!("expected diagnostics after Open, got {other:?}")
            }
        }
    }
}

pub(super) async fn answer(
    events: &mut UnboundedReceiver<Event>,
    wanted: Ticket,
) -> (Answer, Stages) {
    loop {
        if let Event::Answer {
            ticket,
            answer,
            stages,
        } = next(events).await
        {
            if ticket == wanted {
                return (answer, stages);
            }
        }
    }
}

pub(super) fn ask(link: &Link, path: &Path, ticket: u64, ask: Ask) -> Ticket {
    let ticket: Ticket = Ticket(ticket);
    assert!(link.send(Request::Ask {
        ticket,
        path: path.to_path_buf(),
        ask,
        sent: std::time::Instant::now(),
    }));
    ticket
}

pub(super) async fn wait_ready(events: &mut UnboundedReceiver<Event>) -> Vec<(usize, usize)> {
    let mut progress: Vec<(usize, usize)> = Vec::new();
    let started: std::time::Instant = std::time::Instant::now();
    loop {
        let left: Duration = LOAD.saturating_sub(started.elapsed());
        let event: Result<Option<Event>, _> = tokio::time::timeout(left, events.recv()).await;
        let pending: String = format!("still waiting for Ready after {LOAD:?}; {progress:?}");
        match event.expect(&pending).expect("the server thread is alive") {
            Event::Progress { loaded, total } => progress.push((loaded, total)),
            Event::Ready { .. } => return progress,
            Event::Stopped => panic!("server stopped before ready"),
            other @ (Event::Failed(_)
            | Event::Diagnostics { .. }
            | Event::Answer { .. }
            | Event::SyncFailed(_)) => panic!("before ready: {other:?}"),
        }
    }
}
