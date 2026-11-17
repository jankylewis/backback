//! Reading the adapter's events: a stop becomes the call stack and the top
//! frame's variables; output and the end of the program pass through.

use std::path::PathBuf;

use bb_dap::{DapSession, Event, Scope, StackFrame, Variable};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

use super::bb_worker::send;
use super::{Frame, Note, Var};

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
    let exception: Option<String> = if reason == "exception" {
        session
            .exception_info(id)
            .await
            .ok()
            .map(|info: bb_dap::ExceptionInfo| info.headline())
    } else {
        None
    };
    send(
        notes,
        Note::Stopped {
            reason,
            exception,
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
    children(session, scope.variables_reference).await
}

/// The variables under `reference` (a scope, or an opened variable).
pub(super) async fn children(
    session: &DapSession,
    reference: i64,
) -> Result<Vec<Var>, bb_dap::DapError> {
    let vars: Vec<Variable> = session.variables(reference).await?;
    Ok(vars
        .into_iter()
        .map(|v: Variable| Var {
            name: v.name,
            value: v.value,
            reference: v.variables_reference,
        })
        .collect())
}

/// An opened variable's children, or a hovered name's value.
pub(super) async fn inspect(
    session: &DapSession,
    command: super::Command,
    notes: &UnboundedSender<Note>,
) -> Result<(), bb_dap::DapError> {
    match command {
        super::Command::Expand(reference) => {
            let vars: Vec<Var> = children(session, reference).await?;
            send(notes, Note::Children { reference, vars });
        }
        super::Command::Evaluate {
            expression,
            frame,
            token,
        } => {
            let value: Result<Var, String> = evaluate(session, expression, frame).await;
            send(notes, Note::Evaluated { token, value });
        }
        super::Command::Continue
        | super::Command::Step(_)
        | super::Command::Stop
        | super::Command::Frame(_)
        | super::Command::Breakpoints(..)
        | super::Command::Exceptions(_)
        | super::Command::SetNext(..) => {}
    }
    Ok(())
}

/// `expression`'s value in `frame`, as a variable named by it.
async fn evaluate(session: &DapSession, expression: String, frame: i64) -> Result<Var, String> {
    let body: Value = session
        .evaluate(&expression, frame)
        .await
        .map_err(|error: bb_dap::DapError| error.to_string())?;
    let value: String = text(&body, "result").ok_or("no value")?;
    let reference: i64 = body
        .get("variablesReference")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    Ok(Var {
        name: expression,
        value,
        reference,
    })
}

/// How long a debugged test's run may take to write its results.
const RESULTS_GRACE: std::time::Duration = std::time::Duration::from_secs(20);

/// A debugged test's session is over. When the program ran to its end, its
/// results mark its row as a run's do (BB-BUG-066); a Stop leaves none.
pub(super) async fn test_results(
    start: &super::Start,
    (host, folder): (bb_testkit::DebugHost, &bb_process::OwnedTempDir),
    ended: bool,
    notes: &UnboundedSender<Note>,
) {
    if !ended {
        host.stop().await;
        return;
    }
    host.finish(RESULTS_GRACE).await;
    let trx: PathBuf = folder.path().join(crate::tests_panel::DEBUG_TRX);
    let read: Result<Vec<bb_testkit::SpecResult>, String> = std::fs::read_to_string(&trx)
        .map_err(|error: std::io::Error| error.to_string())
        .and_then(|text: String| {
            bb_testkit::parse_trx(&text).map_err(|error: bb_testkit::TrxError| error.to_string())
        });
    match read {
        Ok(results) => send(
            notes,
            Note::TestResults {
                project: start.target.clone(),
                results,
            },
        ),
        Err(why) => send(notes, Note::Output(format!("no test result: {why}\n"))),
    }
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
