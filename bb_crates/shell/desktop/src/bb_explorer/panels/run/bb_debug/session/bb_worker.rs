//! The debug session runs on its own thread, as the C# server does: it
//! builds the target, starts netcoredbg, sends the breakpoints and the
//! exception filters, and reports each stop with its call stack and the top
//! frame's variables. The UI holds only the channel ends, so no handler ever
//! waits on the debugger.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use bb_dap::{DapSession, Event, SourceBreakpoint, Step};
use serde_json::Value;
use tokio::sync::mpsc::{unbounded_channel, Receiver, UnboundedReceiver, UnboundedSender};

use super::{Command, Note, Start, StepKind, Var};

/// Each adapter request's deadline.
const DEADLINE: Duration = Duration::from_secs(30);
/// Each `dotnet` call of the build.
const BUILD: Duration = Duration::from_secs(180);

/// Start the session thread.
pub(crate) fn start(start: Start) -> (UnboundedSender<Command>, UnboundedReceiver<Note>) {
    let (command_tx, commands): (UnboundedSender<Command>, UnboundedReceiver<Command>) =
        unbounded_channel();
    let (notes, note_rx): (UnboundedSender<Note>, UnboundedReceiver<Note>) = unbounded_channel();
    let spawned: std::io::Result<std::thread::JoinHandle<()>> = std::thread::Builder::new()
        .name(String::from("bb-debug"))
        .spawn(move || {
            match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime.block_on(serve(&start, commands, &notes)),
                Err(error) => send(&notes, Note::Failed(format!("debug runtime: {error}"))),
            }
        });
    if let Err(error) = spawned {
        let (failed, failed_rx): (UnboundedSender<Note>, UnboundedReceiver<Note>) =
            unbounded_channel();
        send(&failed, Note::Failed(format!("debug thread: {error}")));
        return (command_tx, failed_rx);
    }
    (command_tx, note_rx)
}

/// Nobody listening means the window closed; nothing to do then.
pub(super) fn send(notes: &UnboundedSender<Note>, note: Note) {
    let _closed: Result<(), tokio::sync::mpsc::error::SendError<Note>> = notes.send(note);
}

async fn serve(start: &Start, commands: UnboundedReceiver<Command>, notes: &UnboundedSender<Note>) {
    send(notes, Note::Building);
    match open(start).await {
        Ok((session, events, host)) => {
            send(notes, Note::CanSetNext(session.can_set_next_statement()));
            send(notes, Note::Running);
            run(&session, events, commands, notes).await;
            if let Some((host, _folder)) = host {
                host.stop().await;
            }
        }
        Err(reason) => send(notes, Note::Failed(reason)),
    }
}

/// A test's paused host and the folder its run writes to.
type Host = (bb_testkit::DebugHost, bb_process::OwnedTempDir);

/// Start the adapter on the target: launch the program, or for a test
/// attach to its paused test host (which then runs on by itself).
async fn open(start: &Start) -> Result<(DapSession, Receiver<Event>, Option<Host>), String> {
    let host: Option<Host> = match &start.test {
        Some((root, fqn)) => {
            Some(crate::tests_panel::debug_host(root.clone(), &start.target, fqn).await?)
        }
        None => None,
    };
    let program: Option<PathBuf> = match &host {
        Some(_) => None,
        None => Some(
            bb_dap::target::resolve(&start.target, BUILD)
                .await
                .map_err(|error: bb_dap::target::TargetError| error.to_string())?,
        ),
    };
    let mut session: DapSession =
        DapSession::spawn(DEADLINE).map_err(|error: bb_dap::DapError| text(&error))?;
    let events: Receiver<Event> = session.take_events().ok_or("no adapter events")?;
    session
        .initialize()
        .await
        .map_err(|error: bb_dap::DapError| text(&error))?;
    match (&host, &program) {
        (Some((paused, _)), _) => session.attach(paused.pid.get()).await,
        (None, Some(program)) => launch(&session, program).await,
        (None, None) => Err(bb_dap::DapError::Closed),
    }
    .map_err(|error: bb_dap::DapError| text(&error))?;
    for (path, lines) in &start.breakpoints {
        set_breakpoints(&session, path, lines)
            .await
            .map_err(|error: bb_dap::DapError| text(&error))?;
    }
    session
        .set_exception_breakpoints(&start.exceptions)
        .await
        .map_err(|error: bb_dap::DapError| text(&error))?;
    session
        .configuration_done()
        .await
        .map_err(|error: bb_dap::DapError| text(&error))?;
    Ok((session, events, host))
}

/// Launch the built `program` under the adapter, in its own folder.
async fn launch(session: &DapSession, program: &Path) -> Result<(), bb_dap::DapError> {
    let cwd: &Path = program.parent().unwrap_or(program);
    session
        .launch(program, &[], cwd, &BTreeMap::new(), true)
        .await
}

fn text(error: &bb_dap::DapError) -> String {
    error.to_string()
}

async fn set_breakpoints(
    session: &DapSession,
    path: &Path,
    lines: &[usize],
) -> Result<Value, bb_dap::DapError> {
    let points: Vec<SourceBreakpoint> = lines
        .iter()
        .map(|line: &usize| SourceBreakpoint {
            line: u32::try_from(line.saturating_add(1)).unwrap_or(u32::MAX),
            condition: None,
        })
        .collect();
    session.set_breakpoints(path, &points).await
}

async fn run(
    session: &DapSession,
    mut events: Receiver<Event>,
    mut commands: UnboundedReceiver<Command>,
    notes: &UnboundedSender<Note>,
) {
    let mut thread: Option<i64> = None;
    loop {
        let going: bool = tokio::select! {
            event = events.recv() => {
                if let Some(event) = event {
                    super::stops::on_event(session, event, &mut thread, notes).await
                } else {
                    send(notes, Note::Ended(String::from("the debugger closed")));
                    false
                }
            }
            command = commands.recv() => match command {
                Some(command) => on_command(session, command, thread, notes).await,
                None => false,
            },
        };
        if !going {
            let _gone: Result<(), bb_dap::DapError> = session.disconnect(true).await;
            return;
        }
    }
}

async fn on_command(
    session: &DapSession,
    command: Command,
    thread: Option<i64>,
    notes: &UnboundedSender<Note>,
) -> bool {
    let resumes: bool = thread.is_some() && matches!(command, Command::Continue | Command::Step(_));
    let done: Result<(), bb_dap::DapError> = match (command, thread) {
        (Command::Stop, _) => {
            send(notes, Note::Ended(String::from("stopped")));
            return false;
        }
        (Command::Breakpoints(path, lines), _) => set_breakpoints(session, &path, &lines)
            .await
            .map(|_: Value| ()),
        (Command::Exceptions(filters), _) => session.set_exception_breakpoints(&filters).await,
        (Command::Frame(frame), _) => super::stops::frame_vars(session, frame)
            .await
            .map(|vars: Vec<Var>| send(notes, Note::Variables { frame, vars })),
        (Command::Continue, Some(id)) => session.continue_thread(id).await,
        (Command::Step(kind), Some(id)) => session.step(step(kind), id).await,
        (Command::SetNext(path, line), Some(id)) => {
            let line: u32 = u32::try_from(line.saturating_add(1)).unwrap_or(u32::MAX);
            if let Err(error) = session.set_next(id, &path, line).await {
                send(notes, Note::Refused(format!("Set Next Statement: {error}")));
            }
            Ok(())
        }
        (Command::Continue | Command::Step(_) | Command::SetNext(..), None) => Ok(()),
    };
    match (done, resumes) {
        (Ok(()), true) => send(notes, Note::Running),
        (Ok(()), false) => {}
        (Err(error), _) => send(notes, Note::Output(format!("debugger: {error}\n"))),
    }
    true
}

fn step(kind: StepKind) -> Step {
    match kind {
        StepKind::Over => Step::Next,
        StepKind::Into => Step::In,
        StepKind::Out => Step::Out,
    }
}
