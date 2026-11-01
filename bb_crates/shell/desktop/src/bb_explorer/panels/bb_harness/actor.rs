use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use bb_mcp::{CallResult, Session, Tool};

use super::model::RowText;
use serde_json::Value;
use tokio::sync::mpsc::{self as async_mpsc, UnboundedReceiver, UnboundedSender};

pub(crate) enum Command {
    Start {
        project: PathBuf,
        generation: u64,
    },
    Stop {
        generation: u64,
    },
    Call {
        name: String,
        arguments: Value,
        generation: u64,
    },
}

pub(crate) enum Event {
    Started {
        tools: Vec<Tool>,
        pid: Option<u32>,
        generation: u64,
    },
    Stopped {
        generation: u64,
    },
    Crashed {
        error: String,
        generation: u64,
    },
    Called {
        text: RowText,
        is_error: bool,
        generation: u64,
    },
    Log {
        line: String,
        generation: u64,
    },
}

pub(crate) struct Actor {
    pub(crate) send: UnboundedSender<Command>,
    pub(crate) receive: Receiver<Event>,
}

struct RuntimeState {
    session: Option<Session>,
    generation: u64,
}

impl Actor {
    pub(crate) fn new() -> Self {
        let (send, commands): (UnboundedSender<Command>, UnboundedReceiver<Command>) =
            async_mpsc::unbounded_channel();
        let (events, receive): (Sender<Event>, Receiver<Event>) = mpsc::channel();
        thread::spawn(move || {
            let Ok(runtime): Result<tokio::runtime::Runtime, std::io::Error> =
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
            else {
                return;
            };
            runtime.block_on(run(commands, events));
        });
        Self { send, receive }
    }
}

async fn run(mut commands: UnboundedReceiver<Command>, events: Sender<Event>) {
    let mut state: RuntimeState = RuntimeState {
        session: None,
        generation: 0,
    };
    loop {
        tokio::select! {
            command = commands.recv() => {
                let Some(command): Option<Command> = command else { break; };
                handle(command, &mut state, &events).await;
            }
            () = tokio::time::sleep(Duration::from_millis(100)) => poll_logs(&mut state, &events),
        }
    }
    if let Some(active) = state.session {
        let _stopped: bool = active.stop().await.is_ok();
    }
}

async fn handle(command: Command, state: &mut RuntimeState, events: &Sender<Event>) {
    match command {
        Command::Start {
            project,
            generation,
        } => start(project, generation, state, events).await,
        Command::Stop { generation } => stop(generation, state, events).await,
        Command::Call {
            name,
            arguments,
            generation,
        } => {
            call(name, arguments, generation, state, events).await;
        }
    }
}

async fn start(
    project: PathBuf,
    generation: u64,
    state: &mut RuntimeState,
    events: &Sender<Event>,
) {
    if let Some(old) = state.session.take() {
        let _stopped: bool = old.stop().await.is_ok();
    }
    state.generation = generation;
    match Session::start(&project).await {
        Ok(mut session) => match session.tools().await {
            Ok(tools) => {
                let pid: Option<u32> = session.pid();
                state.session = Some(session);
                let _sent: bool = events
                    .send(Event::Started {
                        tools,
                        pid,
                        generation,
                    })
                    .is_ok();
            }
            Err(error) => {
                let _stopped: bool = session.stop().await.is_ok();
                let _sent: bool = events
                    .send(Event::Crashed {
                        error: error.to_string(),
                        generation,
                    })
                    .is_ok();
            }
        },
        Err(error) => {
            let _sent: bool = events
                .send(Event::Crashed {
                    error: error.to_string(),
                    generation,
                })
                .is_ok();
        }
    }
}

async fn stop(generation: u64, state: &mut RuntimeState, events: &Sender<Event>) {
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
    let _sent: bool = events.send(event).is_ok();
}

async fn call(
    name: String,
    arguments: Value,
    generation: u64,
    state: &mut RuntimeState,
    events: &Sender<Event>,
) {
    if generation != state.generation {
        return;
    }
    let Some(session): Option<&mut Session> = state.session.as_mut() else {
        return;
    };
    let event: Event = match session.call(&name, arguments).await {
        Ok(result) => called(&result, generation),
        Err(error) => Event::Crashed {
            error: error.to_string(),
            generation,
        },
    };
    let crashed: bool = matches!(event, Event::Crashed { .. });
    let _sent: bool = events.send(event).is_ok();
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

fn poll_logs(state: &mut RuntimeState, events: &Sender<Event>) {
    if let Some(active) = state.session.as_mut() {
        while let Ok(line) = active.logs().try_recv() {
            let _sent: bool = events
                .send(Event::Log {
                    line,
                    generation: state.generation,
                })
                .is_ok();
        }
    }
}
