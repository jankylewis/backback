use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant};

use bb_dap::{DapError, DapSession, Event, SourceBreakpoint};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{mpsc, OnceCell};

use crate::fixture;

pub const DEADLINE: Duration = Duration::from_secs(15);
static BUILT: OnceCell<fixture::Built> = OnceCell::const_new();

pub type Events = mpsc::Receiver<Event>;

pub async fn built() -> &'static fixture::Built {
    BUILT.get_or_init(|| fixture::build("lifecycle")).await
}

pub async fn dll() -> PathBuf {
    built().await.dir.join("bin/Debug/net10.0/GotoFixture.dll")
}

pub async fn spin_line() -> (PathBuf, u32) {
    let source: PathBuf = built().await.dir.join("Lifecycle.cs");
    let text: String = std::fs::read_to_string(&source).unwrap();
    let index: usize = text
        .lines()
        .position(|line: &str| line.trim() == "int next = tick + 1;")
        .unwrap();
    (source, u32::try_from(index + 1).unwrap())
}

pub async fn start_session() -> (DapSession, Events) {
    bb_dap::locate().expect("netcoredbg must be installed for this gate");
    let mut session: DapSession = DapSession::spawn(DEADLINE).unwrap();
    let events: Events = session.take_events().unwrap();
    session.initialize().await.unwrap();
    (session, events)
}

pub async fn launch(mode: &str, condition: Option<&str>) -> (DapSession, Events) {
    let (session, events): (DapSession, Events) = start_session().await;
    let program: PathBuf = dll().await;
    let cwd: PathBuf = built().await.dir.clone();
    let args: Vec<String> = vec![mode.to_owned()];
    session
        .launch(&program, &args, &cwd, &BTreeMap::new(), false)
        .await
        .unwrap();
    let (source, line): (PathBuf, u32) = spin_line().await;
    let point: SourceBreakpoint = SourceBreakpoint {
        line,
        condition: condition.map(str::to_owned),
    };
    session.set_breakpoints(&source, &[point]).await.unwrap();
    session.configuration_done().await.unwrap();
    (session, events)
}

pub struct Target {
    pub child: Child,
    stdin: ChildStdin,
}

impl Target {
    pub async fn start() -> Self {
        let mut child: Child = Command::new("dotnet")
            .arg(dll().await)
            .arg("await")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        let stdin: ChildStdin = child.stdin.take().unwrap();
        let stdout: ChildStdout = child.stdout.take().unwrap();
        let mut lines: tokio::io::Lines<BufReader<ChildStdout>> = BufReader::new(stdout).lines();
        let ready: Option<String> = tokio::time::timeout(DEADLINE, lines.next_line())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(ready.as_deref(), Some("ready"));
        tokio::spawn(async move { while let Ok(Some(_)) = lines.next_line().await {} });
        Self { child, stdin }
    }

    pub fn pid(&self) -> u32 {
        self.child.id().unwrap()
    }

    pub async fn release(&mut self) -> std::process::ExitStatus {
        self.stdin.write_all(b"go\n").await.unwrap();
        self.stdin.flush().await.unwrap();
        tokio::time::timeout(DEADLINE, self.child.wait())
            .await
            .unwrap()
            .unwrap()
    }
}

pub struct Stop {
    pub event: Event,
    pub thread: i64,
    pub process: Option<u32>,
}

pub async fn first_stop(events: &mut Events) -> Stop {
    let mut process: Option<u32> = None;
    loop {
        let event: Event = DapSession::next_event(events, DEADLINE).await.unwrap();
        let body: Option<&Value> = match &event {
            Event::Other(name, body) if name == "process" => {
                process = body
                    .get("systemProcessId")
                    .and_then(Value::as_u64)
                    .and_then(|pid: u64| u32::try_from(pid).ok());
                None
            }
            Event::Stopped(body) | Event::ConditionError { stopped: body, .. } => Some(body),
            Event::Other(_, _)
            | Event::Output(_)
            | Event::Terminated(_)
            | Event::Exited(_)
            | Event::Breakpoint(_) => None,
        };
        if let Some(body) = body {
            let thread: i64 = body.get("threadId").and_then(Value::as_i64).unwrap();
            return Stop {
                event,
                thread,
                process,
            };
        }
    }
}

pub async fn drain_to_error(events: &mut Events) -> DapError {
    loop {
        if let Err(error) = DapSession::next_event(events, DEADLINE).await {
            return error;
        }
    }
}

pub async fn signal(sig: &str, pid: u32) -> bool {
    tokio::time::timeout(
        DEADLINE,
        Command::new("kill")
            .arg(sig)
            .arg(pid.to_string())
            .stderr(Stdio::null())
            .status(),
    )
    .await
    .unwrap()
    .unwrap()
    .success()
}

pub async fn gone_within(pid: u32, limit: Duration) -> bool {
    let started: Instant = Instant::now();
    while started.elapsed() < limit {
        if !signal("-0", pid).await {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    !signal("-0", pid).await
}
