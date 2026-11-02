//! Reading the adapter's events: a stop becomes the call stack and the top
//! frame's variables; output and the end of the program pass through.

use std::path::PathBuf;

use bb_dap::{DapSession, Event, Scope, StackFrame, Variable};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

use super::bb_worker::{send, Frame, Note, Var};

/// One adapter event; `false` when the session is over.
pub(super) async fn on_event(
    session: &DapSession,
    event: Event,
    thread: &mut Option<i64>,
    notes: &UnboundedSender<Note>,
) -> bool {
    match event {
        Event::Stopped(body) => {
            let reason: String = text(&body, "reason").unwrap_or_else(|| String::from("pause"));
            stopped(session, &body, reason, thread, notes).await;
        }
        Event::ConditionError {
            message,
            stopped: body,
        } => {
            send(
                notes,
                Note::Output(format!("breakpoint condition: {message}\n")),
            );
            stopped(
                session,
                &body,
                String::from("condition error"),
                thread,
                notes,
            )
            .await;
        }
        Event::Output(body) => {
            if let Some(output) = text(&body, "output") {
                send(notes, Note::Output(output));
            }
        }
        Event::Exited(body) => exited(&body, notes),
        Event::Terminated(_) => {
            send(notes, Note::Ended(String::from("the program ended")));
            return false;
        }
        Event::Breakpoint(_) | Event::Other(_, _) => {}
    }
    true
}

fn exited(body: &Value, notes: &UnboundedSender<Note>) {
    let code: String = body
        .get("exitCode")
        .and_then(Value::as_i64)
        .map_or_else(|| String::from("?"), |code: i64| code.to_string());
    send(
        notes,
        Note::Output(format!("program exited with code {code}\n")),
    );
}

async fn stopped(
    session: &DapSession,
    body: &Value,
    reason: String,
    thread: &mut Option<i64>,
    notes: &UnboundedSender<Note>,
) {
    let Some(id): Option<i64> = body.get("threadId").and_then(Value::as_i64) else {
        return;
    };
    *thread = Some(id);
    let frames: Vec<Frame> = match session.stack_trace(id).await {
        Ok(frames) => frames.iter().map(frame).collect(),
        Err(error) => {
            send(notes, Note::Failed(format!("call stack: {error}")));
            return;
        }
    };
    let vars: Vec<Var> = match frames.first() {
        Some(top) => frame_vars(session, top.id).await.unwrap_or_default(),
        None => Vec::new(),
    };
    send(
        notes,
        Note::Stopped {
            reason,
            frames,
            vars,
        },
    );
}

/// The first scope's variables of `frame` (the locals and arguments).
pub(super) async fn frame_vars(
    session: &DapSession,
    frame: i64,
) -> Result<Vec<Var>, bb_dap::DapError> {
    let scopes: Vec<Scope> = session.scopes(frame).await?;
    let Some(scope): Option<&Scope> = scopes.first() else {
        return Ok(Vec::new());
    };
    let vars: Vec<Variable> = session.variables(scope.variables_reference).await?;
    Ok(vars
        .into_iter()
        .map(|v: Variable| Var {
            name: v.name,
            value: v.value,
        })
        .collect())
}

fn frame(raw: &StackFrame) -> Frame {
    let path: Option<PathBuf> = raw
        .source
        .as_ref()
        .and_then(|source: &Value| source.get("path"))
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .filter(|path: &PathBuf| path.is_file());
    Frame {
        id: raw.id,
        name: raw.name.clone(),
        path,
        line: usize::try_from(raw.line.saturating_sub(1)).unwrap_or(0),
    }
}

fn text(body: &Value, key: &str) -> Option<String> {
    body.get(key).and_then(Value::as_str).map(str::to_owned)
}
