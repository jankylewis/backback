//! Confirm cleanup of a launched debuggee in the shared process group.

use std::process::{ExitStatus, Stdio};
use std::time::{Duration, Instant};

use bb_process::GroupId;

use crate::Debuggee;

const DEADLINE: Duration = Duration::from_secs(1);
const TICK: Duration = Duration::from_millis(10);

pub(super) async fn end(group: GroupId) -> Debuggee {
    tokio::task::spawn_blocking(move || end_blocking(group))
        .await
        .unwrap_or(Debuggee::Unconfirmed)
}

fn end_blocking(group: GroupId) -> Debuggee {
    group.end(&mut || {});
    let started: Instant = Instant::now();
    while started.elapsed() < DEADLINE {
        if !group.alive() {
            return Debuggee::Ended;
        }
        std::thread::sleep(TICK);
    }
    Debuggee::Unconfirmed
}

pub(super) async fn alive(pid: u32) -> bool {
    let check: Result<Result<ExitStatus, std::io::Error>, tokio::time::error::Elapsed> =
        tokio::time::timeout(
            DEADLINE,
            tokio::process::Command::new("/bin/kill")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_group_is_ended_at_once() {
        let started: Instant = Instant::now();
        assert_eq!(
            end_blocking(GroupId::register(
                std::num::NonZeroU32::new(2_147_483_000).unwrap()
            )),
            Debuggee::Ended
        );
        assert!(started.elapsed() < DEADLINE);
    }

    #[tokio::test]
    async fn own_process_is_alive_and_a_free_pid_is_not() {
        assert!(alive(std::process::id()).await);
        assert!(!alive(2_147_483_000).await);
    }
}
