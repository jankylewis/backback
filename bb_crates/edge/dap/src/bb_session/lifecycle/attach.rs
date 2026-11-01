//! Attach target watch: netcoredbg sends no terminal event when the attached
//! process exits before `configurationDone`, so the session polls the pid.

use std::time::Duration;

use serde_json::json;
use tokio::sync::mpsc;

use crate::Event;

use super::{group, Liveness};

/// Poll period for the target pid.
const POLL: Duration = Duration::from_millis(200);
/// Time the adapter gets to send its own `terminated` after the target exits.
const GRACE: Duration = Duration::from_millis(250);

/// Watch `pid`; when it is gone and the adapter sent no `terminated`, emit one.
///
/// The event body is `{"backback": "attach target exited", "processId": pid}`.
/// The task holds only a weak sender, so it never keeps the stream open.
pub(in crate::bb_session) fn watch_target(
    pid: u32,
    events: mpsc::WeakSender<Event>,
    live: Liveness,
) {
    tokio::spawn(async move {
        loop {
            if live.is_closed() || live.is_terminated() {
                return;
            }
            if !group::alive(pid).await {
                break;
            }
            tokio::time::sleep(POLL).await;
        }
        tokio::time::sleep(GRACE).await;
        if live.is_closed() || live.is_terminated() {
            return;
        }
        if let Some(sender) = events.upgrade() {
            let body: serde_json::Value =
                json!({"backback": "attach target exited", "processId": pid});
            if sender.send(Event::Terminated(body)).await.is_ok() {
                live.saw_terminated();
            }
        }
    });
}
