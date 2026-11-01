//! Run the real `bb` binary with deadlines and capture what it printed.
//!
//! A Roslyn command (`open`, `probe`) is not held to one flat wall: under
//! load the solution takes ~42 s to load (load ~10). It waits on the
//! readiness line `bb` prints once every distinct project is loaded, with a
//! generous outer deadline, then a request budget for the rest.

use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

/// The line `bb open` and `bb probe` print when the solution is loaded.
pub const READY_LINE: &str = "loaded    : ";
/// Any other `bb` call must end within this.
const QUICK: Duration = Duration::from_secs(30);
/// Outlasts `bb`'s own 300 s load deadline, so its typed timeout (naming
/// the missing projects) is what a stuck load reports.
const LOAD: Duration = Duration::from_secs(330);
/// After readiness; `bb`'s own deadline per request is 60 s.
const AFTER_READY: Duration = Duration::from_secs(120);

/// What one `bb` run produced.
pub struct Run {
    /// Exit code (`None` when killed by a signal).
    pub code: Option<i32>,
    /// Standard output.
    pub out: String,
    /// Standard error.
    pub err: String,
}

impl Run {
    /// Both streams, for "never printed anywhere" checks.
    pub fn all(&self) -> String {
        format!("{}{}", self.out, self.err)
    }
}

/// How long a run may take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    /// One deadline for the whole run.
    Flat(Duration),
    /// `ready` until [`READY_LINE`], then `after` until the exit.
    UntilReady { ready: Duration, after: Duration },
}

/// What a run was still waiting for when its deadline passed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pending {
    /// The readiness line: some project of the solution was not loaded.
    Ready,
    /// The exit: the command (or a request after readiness) did not end.
    Exit,
}

/// A run killed at its deadline.
#[derive(Debug)]
pub struct Stalled {
    pub pending: Pending,
    pub after: Duration,
    pub out: String,
    pub err: String,
}

impl std::fmt::Display for Stalled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let what: &str = match self.pending {
            Pending::Ready => "the readiness line `loaded    :` (every distinct project loaded)",
            Pending::Exit => "the exit",
        };
        write!(
            f,
            "still waiting for {what} after {:?}\n--- stdout so far\n{}--- stderr so far\n{}",
            self.after, self.out, self.err
        )
    }
}

/// The `_Harness` fixture solution folder.
pub fn harness() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../desktop/fixtures/_Harness")
}

fn plan_for(args: &[&str]) -> Plan {
    match args.first() {
        Some(&"open" | &"probe") => Plan::UntilReady {
            ready: LOAD,
            after: AFTER_READY,
        },
        _ => Plan::Flat(QUICK),
    }
}

/// Run `bb` with `args`; kill it and fail the test at its deadline.
pub fn bb(args: &[&str]) -> Run {
    bb_env(args, &[])
}

/// Run `bb` with `args` and extra environment variables; waits on the
/// command's own readiness lines, never a fixed wall (BBEOD03-LOADCOUNT).
pub fn bb_env(args: &[&str], env: &[(&str, &str)]) -> Run {
    let child: Child = Command::new(env!("CARGO_BIN_EXE_bb"))
        .args(args)
        .envs(env.iter().copied())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    watch(child, plan_for(args)).unwrap_or_else(|stalled: Stalled| panic!("bb {args:?}: {stalled}"))
}

enum Line {
    Out(String),
    Err(String),
}

fn forward(pipe: impl Read + Send + 'static, tx: Sender<Line>, wrap: fn(String) -> Line) {
    std::thread::spawn(move || {
        for line in BufReader::new(pipe).lines().map_while(Result::ok) {
            if tx.send(wrap(line)).is_err() {
                break;
            }
        }
    });
}

struct Seen {
    out: String,
    err: String,
    ready_at: Option<Instant>,
}

impl Seen {
    fn take(&mut self, rx: &Receiver<Line>) {
        while let Ok(line) = rx.try_recv() {
            self.push(line);
        }
    }

    fn push(&mut self, line: Line) {
        match line {
            Line::Out(text) => {
                if self.ready_at.is_none() && text.starts_with(READY_LINE) {
                    self.ready_at = Some(Instant::now());
                }
                self.out.push_str(&text);
                self.out.push('\n');
            }
            Line::Err(text) => {
                self.err.push_str(&text);
                self.err.push('\n');
            }
        }
    }
}

fn overdue(plan: Plan, started: Instant, ready_at: Option<Instant>) -> Option<(Pending, Duration)> {
    let (pending, since, limit): (Pending, Instant, Duration) = match (plan, ready_at) {
        (Plan::Flat(limit), _) => (Pending::Exit, started, limit),
        (Plan::UntilReady { ready, .. }, None) => (Pending::Ready, started, ready),
        (Plan::UntilReady { after, .. }, Some(at)) => (Pending::Exit, at, after),
    };
    (since.elapsed() > limit).then_some((pending, limit))
}

/// Wait for `child` under `plan`, collecting its output line by line.
pub fn watch(mut child: Child, plan: Plan) -> Result<Run, Stalled> {
    let (tx, rx): (Sender<Line>, Receiver<Line>) = channel();
    forward(child.stdout.take().unwrap(), tx.clone(), Line::Out);
    forward(child.stderr.take().unwrap(), tx, Line::Err);
    let started: Instant = Instant::now();
    let mut seen: Seen = Seen {
        out: String::new(),
        err: String::new(),
        ready_at: None,
    };
    loop {
        seen.take(&rx);
        if let Some(status) = child.try_wait().unwrap() {
            return Ok(finish(status, seen, &rx));
        }
        if let Some((pending, after)) = overdue(plan, started, seen.ready_at) {
            child.kill().unwrap();
            let _reaped: std::io::Result<ExitStatus> = child.wait();
            seen.take(&rx);
            return Err(Stalled {
                pending,
                after,
                out: seen.out,
                err: seen.err,
            });
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// The child exited: read the rest until both pipes close.
fn finish(status: ExitStatus, mut seen: Seen, rx: &Receiver<Line>) -> Run {
    // Ends when both pipes close; a grandchild holding one open is cut off.
    while let Ok(line) = rx.recv_timeout(Duration::from_secs(5)) {
        seen.push(line);
    }
    Run {
        code: status.code(),
        out: seen.out,
        err: seen.err,
    }
}
