//! One thread per server: a small tokio runtime that owns the `Session`.
//! Slow work (Start, a tool call) runs interruptibly, so Stop and Start are
//! never queued behind it.

#[path = "bb_work.rs"]
mod work;

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use bb_mcp::{Session, Tool};
use serde_json::Value;
use tokio::sync::mpsc::{self as async_mpsc, UnboundedReceiver, UnboundedSender};

use super::model::RowText;

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

/// Commands that arrive while slow work runs.
struct Inbox {
    commands: UnboundedReceiver<Command>,
    pending: VecDeque<Command>,
}

impl Inbox {
    /// Run `work` until it ends or a Stop/Start arrives. On Stop/Start the
    /// work is dropped (cancelled) and `None` returns; that command runs
    /// next. A Call waits its turn. A closed channel also cancels.
    async fn interruptible<F: std::future::Future>(&mut self, work: F) -> Option<F::Output> {
        tokio::pin!(work);
        loop {
            tokio::select! {
                done = &mut work => return Some(done),
                next = self.commands.recv() => match next {
                    Some(command @ (Command::Stop { .. } | Command::Start { .. })) => {
                        self.pending.push_front(command);
                        return None;
                    }
                    Some(command @ Command::Call { .. }) => self.pending.push_back(command),
                    None => return None,
                },
            }
        }
    }

    /// The next command, or `None` once the harness is gone.
    async fn next(&mut self, state: &mut RuntimeState, events: &Sender<Event>) -> Option<Command> {
        loop {
            if let Some(command) = self.pending.pop_front() {
                return Some(command);
            }
            tokio::select! {
                command = self.commands.recv() => return command,
                () = tokio::time::sleep(Duration::from_millis(100)) => {
                    work::poll_logs(state, events);
                }
            }
        }
    }
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

async fn run(commands: UnboundedReceiver<Command>, events: Sender<Event>) {
    let mut state: RuntimeState = RuntimeState {
        session: None,
        generation: 0,
    };
    let mut inbox: Inbox = Inbox {
        commands,
        pending: VecDeque::new(),
    };
    while let Some(command) = inbox.next(&mut state, &events).await {
        match command {
            Command::Start {
                project,
                generation,
            } => work::start(&project, generation, &mut state, &events, &mut inbox).await,
            Command::Stop { generation } => work::stop(generation, &mut state, &events).await,
            Command::Call {
                name,
                arguments,
                generation,
            } => {
                let call: work::Call = work::Call {
                    name,
                    arguments,
                    generation,
                };
                work::call(call, &mut state, &events, &mut inbox).await;
            }
        }
    }
    if let Some(active) = state.session {
        let _stopped: bool = active.stop().await.is_ok();
    }
}
