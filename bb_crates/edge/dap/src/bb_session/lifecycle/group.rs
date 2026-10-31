//! Process checks and process-group cleanup through `/bin/kill`, each bounded.
//!
//! The adapter starts in its own process group, so a debuggee it launches
//! shares that group. An attached process never does, so it is never hit.

use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use crate::Debuggee;

const KILL: &str = "/bin/kill";
const DEADLINE: Duration = Duration::from_secs(1);
const TICK: Duration = Duration::from_millis(10);

/// End every process left in the group `pgid`; confirm the group is empty.
///
/// Blocks for at most about two seconds.
pub(in crate::bb_session) fn end_blocking(pgid: u32) -> Debuggee {
    let group: String = format!("-{pgid}");
    let _signalled: Option<bool> = kill_group(pgid);
    let started: Instant = Instant::now();
    while started.elapsed() < DEADLINE {
        if run(&["-0", "--", &group]) == Some(false) {
            return Debuggee::Ended;
        }
        std::thread::sleep(TICK);
    }
    Debuggee::Unconfirmed
}

/// Send `SIGKILL` to every process in the group `pgid`, without waiting for
/// them to go. `Some(true)` when a process received it.
pub(in crate::bb_session) fn kill_group(pgid: u32) -> Option<bool> {
    run(&["-KILL", "--", &format!("-{pgid}")])
}

/// Async form of [`end_blocking`], off the runtime threads.
pub(super) async fn end(pgid: u32) -> Debuggee {
    tokio::task::spawn_blocking(move || end_blocking(pgid))
        .await
        .unwrap_or(Debuggee::Unconfirmed)
}

/// Whether `pid` still exists. Unknown (the check failed) counts as alive.
pub(super) async fn alive(pid: u32) -> bool {
    let check: Result<Result<ExitStatus, std::io::Error>, tokio::time::error::Elapsed> =
        tokio::time::timeout(
            DEADLINE,
            tokio::process::Command::new(KILL)
                .arg("-0")
                .arg(pid.to_string())
                .stderr(Stdio::null())
                .kill_on_drop(true)
                .status(),
        )
        .await;
    match check {
        Ok(Ok(status)) => status.success(),
        Ok(Err(_)) | Err(_) => true,
    }
}

/// Run `/bin/kill` with a deadline: `Some(success)`, or `None` on failure.
fn run(args: &[&str]) -> Option<bool> {
    let mut child: Child = Command::new(KILL)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let started: Instant = Instant::now();
    while started.elapsed() < DEADLINE {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status.success()),
            Ok(None) => std::thread::sleep(Duration::from_millis(1)),
            Err(_) => return None,
        }
    }
    let _killed: std::io::Result<()> = child.kill();
    let _reaped: std::io::Result<ExitStatus> = child.wait();
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_group_is_ended_at_once() {
        let started: Instant = Instant::now();
        assert_eq!(end_blocking(2_147_483_000), Debuggee::Ended);
        assert!(started.elapsed() < DEADLINE);
    }

    #[tokio::test]
    async fn own_process_is_alive_and_a_free_pid_is_not() {
        assert!(alive(std::process::id()).await);
        assert!(!alive(2_147_483_000).await);
    }
}
