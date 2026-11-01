//! Kill the real Roslyn child and verify the stopped signal and new session.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use tokio::sync::mpsc::UnboundedReceiver;

use super::{answer, ask, first_diagnostics, loaded, next, start, wait_ready, Live, CLASS_USE};
use crate::intel::bb_worker::protocol::{Answer, Ask, Event, Request, Ticket};
use crate::intel::bb_worker::Link;

#[tokio::test]
async fn a_dead_server_stops_within_one_second_and_restart_answers_again() {
    let mut live: Live = loaded("restart").await;
    let before: Instant = Instant::now();
    assert!(live.link.send(Request::KillForTest));
    let stopped: Event = tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            if let event @ Event::Stopped = next(&mut live.events).await {
                break event;
            }
        }
    })
    .await
    .expect("stopped within one second");
    assert_eq!(stopped, Event::Stopped);
    assert!(before.elapsed() < Duration::from_secs(1));
    let solution: PathBuf = live.root.join("Harness.slnx");
    let (link, mut events): (Link, UnboundedReceiver<Event>) = start(&solution);
    wait_ready(&mut events).await;
    let text: String = std::fs::read_to_string(&live.spec).unwrap();
    assert!(link.send(Request::Open {
        path: live.spec.clone(),
        text,
    }));
    first_diagnostics(&mut events).await;
    let ticket: Ticket = ask(&link, &live.spec, 1, Ask::Completion(CLASS_USE));
    assert!(matches!(
        answer(&mut events, ticket).await.0,
        Answer::Completion(_)
    ));
}
