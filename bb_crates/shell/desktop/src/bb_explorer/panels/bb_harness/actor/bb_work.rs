//! The actor's slow work: start, stop and call one server.

use std::path::Path;
use std::sync::mpsc::Sender;

use bb_mcp::{CallResult, McpError, Session, Tool};
use serde_json::Value;

use super::{Event, Inbox, RuntimeState};
use crate::harness::model::RowText;

pub(super) struct Call {
    pub(super) name: String,
    pub(super) arguments: Value,
    pub(super) generation: u64,
}

fn send(events: &Sender<Event>, event: Event) {
    // The harness may be gone (window closed); then nobody needs the event.
    let _sent: bool = events.send(event).is_ok();
}

/// Build, launch and list tools. Dropping this future (Stop or a new Start)
/// drops the half-made `Session`, whose drop ends the server group.
async fn open(project: &Path) -> Result<(Session, Vec<Tool>), McpError> {
    let mut session: Session = Session::start(project).await?;
    match session.tools().await {
        Ok(tools) => Ok((session, tools)),
        Err(error) => {
            let _stopped: bool = session.stop().await.is_ok();
            Err(error)
        }
    }
}

pub(super) async fn start(
    project: &Path,
    generation: u64,
    state: &mut RuntimeState,
    events: &Sender<Event>,
    inbox: &mut Inbox,
) {
    if let Some(old) = state.session.take() {
        let _stopped: bool = old.stop().await.is_ok();
    }
    state.generation = generation;
    let Some(opened): Option<Result<(Session, Vec<Tool>), McpError>> =
        inbox.interruptible(open(project)).await
    else {
        return;
    };
    match opened {
        Ok((session, tools)) => {
            let pid: Option<u32> = session.pid();
            state.session = Some(session);
            send(
                events,
                Event::Started {
                    tools,
                    pid,
                    generation,
                },
            );
        }
        Err(error) => send(
            events,
            Event::Crashed {
                error: error.to_string(),
                generation,
            },
        ),
    }
}

pub(super) async fn stop(generation: u64, state: &mut RuntimeState, events: &Sender<Event>) {
    state.generation = generation;
    let event: Event = if let Some(old) = state.session.take() {
        match old.stop().await {
            Ok(_status) => Event::Stopped { generation },
            Err(error) => Event::Crashed {
                error: error.to_string(),
                generation,
            },
        }
    } else {
        Event::Stopped { generation }
    };
    send(events, event);
}

pub(super) async fn call(
    call: Call,
    state: &mut RuntimeState,
    events: &Sender<Event>,
    inbox: &mut Inbox,
) {
    if call.generation != state.generation {
        return;
    }
    let Some(session): Option<&mut Session> = state.session.as_mut() else {
        return;
    };
    let Some(outcome): Option<Result<CallResult, McpError>> = inbox
        .interruptible(session.call(&call.name, call.arguments))
        .await
    else {
        return;
    };
    let event: Event = match outcome {
        Ok(result) => called(&result, call.generation),
        Err(error) => Event::Crashed {
            error: error.to_string(),
            generation: call.generation,
        },
    };
    let crashed: bool = matches!(event, Event::Crashed { .. });
    send(events, event);
    if crashed {
        if let Some(old) = state.session.take() {
            let _stopped: bool = old.stop().await.is_ok();
        }
    }
}

/// Pretty-print and cut into rows here, off the UI thread.
fn called(result: &CallResult, generation: u64) -> Event {
    let pretty: String = serde_json::to_string_pretty(&result.content)
        .unwrap_or_else(|error: serde_json::Error| format!("cannot show result: {error}"));
    Event::Called {
        text: RowText::from(pretty),
        is_error: result.is_error,
        generation,
    }
}

pub(super) fn poll_logs(state: &mut RuntimeState, events: &Sender<Event>) {
    if let Some(active) = state.session.as_mut() {
        while let Ok(line) = active.logs().try_recv() {
            send(
                events,
                Event::Log {
                    line,
                    generation: state.generation,
                },
            );
        }
    }
}
