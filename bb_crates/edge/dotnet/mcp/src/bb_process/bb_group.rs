//! Own each server's process group: no member may outlive its owner.
//!
//! Every server starts as the leader of its own process group. The group is
//! recorded here while its `Session` lives. A `Session` drop, a fatal signal
//! and an app quit all end the group the same way: TERM, a short grace, KILL.

use std::process::{Command, Stdio};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use tokio::signal::unix::{signal, Signal, SignalKind};
use tokio::sync::mpsc;

/// Time a group gets to exit after TERM before KILL.
pub(crate) const GRACE: Duration = Duration::from_millis(200);
/// Deadline for one `/bin/kill` call.
const KILL_DEADLINE: Duration = Duration::from_millis(500);
const POLL: Duration = Duration::from_millis(5);

static LIVE: Mutex<Vec<GroupId>> = Mutex::new(Vec::new());

/// The id of a server's process group (its leader's pid).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GroupId(u32);

impl GroupId {
    /// Record the group that `leader` leads; it stays live until [`Self::end`].
    pub(crate) fn register(leader: u32) -> Self {
        let group: Self = Self(leader);
        live().push(group);
        group
    }

    /// Whether any member (a zombie leader too) still exists.
    fn alive(self) -> bool {
        self.send("-0")
    }

    /// Send `signal` to every member; `false` when no member got it.
    fn send(self, signal: &str) -> bool {
        let spawned: std::io::Result<std::process::Child> = Command::new("/bin/kill")
            .args([signal, "--", &format!("-{}", self.0)])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        let Ok(mut kill): std::io::Result<std::process::Child> = spawned else {
            return false;
        };
        let deadline: Instant = Instant::now() + KILL_DEADLINE;
        loop {
            match kill.try_wait() {
                Ok(Some(status)) => return status.success(),
                Ok(None) if Instant::now() < deadline => std::thread::sleep(POLL),
                Ok(None) | Err(_) => {
                    // A kill that hangs past its deadline is stopped; the
                    // caller then treats the group as not signalled.
                    let _stopped: std::io::Result<()> = kill.kill();
                    let _reaped: std::io::Result<std::process::ExitStatus> = kill.wait();
                    return false;
                }
            }
        }
    }

    /// End the group now: TERM, up to [`GRACE`], then KILL; then forget it.
    /// `reap` collects the leader's exit so a zombie does not read as alive.
    pub(crate) fn end(self, reap: &mut dyn FnMut()) {
        reap();
        if self.alive() {
            self.send("-TERM");
            let deadline: Instant = Instant::now() + GRACE;
            loop {
                reap();
                if !self.alive() {
                    break;
                }
                if Instant::now() >= deadline {
                    self.send("-KILL");
                    break;
                }
                std::thread::sleep(POLL);
            }
        }
        live().retain(|group: &Self| *group != self);
    }
}

fn live() -> MutexGuard<'static, Vec<GroupId>> {
    LIVE.lock().unwrap_or_else(PoisonError::into_inner)
}

/// End every live server group now (fatal signal or app quit).
pub fn end_all_servers() {
    let groups: Vec<GroupId> = live().clone();
    for group in groups {
        group.end(&mut || {});
    }
}

/// On SIGTERM or SIGHUP (and SIGINT when `interrupt`), end every live server
/// group, then exit with `128 + signal`. The watcher runs on its own thread.
///
/// # Errors
/// The signal handlers cannot be installed.
pub fn exit_on_signal(interrupt: bool) -> std::io::Result<()> {
    let runtime: tokio::runtime::Runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let mut kinds: Vec<(SignalKind, i32)> =
        vec![(SignalKind::terminate(), 15), (SignalKind::hangup(), 1)];
    if interrupt {
        kinds.push((SignalKind::interrupt(), 2));
    }
    // `enter`, not `block_on`: the caller may already run inside a runtime.
    let entered: tokio::runtime::EnterGuard<'_> = runtime.enter();
    let streams: std::io::Result<Vec<(Signal, i32)>> = kinds
        .into_iter()
        .map(|(kind, number): (SignalKind, i32)| signal(kind).map(|s: Signal| (s, number)))
        .collect();
    drop(entered);
    let streams: Vec<(Signal, i32)> = streams?;
    std::thread::spawn(move || {
        let number: i32 = runtime.block_on(first_signal(streams));
        end_all_servers();
        std::process::exit(number.saturating_add(128));
    });
    Ok(())
}

async fn first_signal(streams: Vec<(Signal, i32)>) -> i32 {
    let (sender, mut receiver): (mpsc::UnboundedSender<i32>, mpsc::UnboundedReceiver<i32>) =
        mpsc::unbounded_channel();
    for (mut stream, number) in streams {
        let sender: mpsc::UnboundedSender<i32> = sender.clone();
        tokio::spawn(async move {
            if stream.recv().await.is_some() {
                let _sent: Result<(), mpsc::error::SendError<i32>> = sender.send(number);
            }
        });
    }
    drop(sender);
    receiver.recv().await.unwrap_or(15)
}
