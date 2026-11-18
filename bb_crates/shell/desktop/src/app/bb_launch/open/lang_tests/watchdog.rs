//! A wait in a window test that ends the run loudly at its deadline, even
//! while the test thread is blocked: `run_until_parked` runs a background
//! job (real cargo or dotnet) to its end, and a deadline checked between
//! those calls cannot fire while one runs (BB-GAT-010).

use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

/// Ends the test run when not dropped within its deadline; dropping it in
/// time disarms it.
pub(crate) struct Watchdog {
    _disarm: Sender<()>,
}

/// Arm a watchdog for `what`, `within` from now: past it, the run prints
/// `waited <within> for <what>` and exits non-zero (no other way ends a
/// test blocked inside `run_until_parked`).
pub(crate) fn watchdog(what: &str, within: Duration) -> Watchdog {
    let (disarm, armed): (Sender<()>, Receiver<()>) = channel();
    let what: String = what.to_owned();
    let _watch: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        if armed.recv_timeout(within) == Err(RecvTimeoutError::Timeout) {
            eprintln!("waited {within:?} for {what}: the test is blocked; ending the run");
            std::process::exit(101);
        }
    });
    Watchdog { _disarm: disarm }
}

#[cfg(test)]
mod tests {
    use std::process::{Child, Command, Stdio};
    use std::time::{Duration, Instant};

    const PROBE: &str = "BB_WATCHDOG_PROBE";
    const NAME: &str = "launch::open::lang_tests::watchdog::tests::a_blocked_wait_ends_the_run_at_its_deadline_naming_what_it_waited_for";

    fn blocked_probe() {
        let _armed: super::Watchdog = super::watchdog("the probe", Duration::from_secs(1));
        std::thread::sleep(Duration::from_secs(30));
    }

    #[test]
    fn a_blocked_wait_ends_the_run_at_its_deadline_naming_what_it_waited_for() {
        if std::env::var_os(PROBE).is_some() {
            blocked_probe();
        } else {
            the_probe_run_ends_loudly();
        }
    }

    fn the_probe_run_ends_loudly() {
        let mut child: Child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", NAME, "--test-threads=1", "--nocapture"])
            .env(PROBE, "1")
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let started: Instant = Instant::now();
        while child.try_wait().unwrap().is_none() && started.elapsed() < Duration::from_secs(15) {
            std::thread::sleep(Duration::from_millis(50));
        }
        let ended: bool = child.try_wait().unwrap().is_some();
        if !ended {
            child.kill().unwrap();
        }
        let output: std::process::Output = child.wait_with_output().unwrap();
        let said: String = String::from_utf8_lossy(&output.stderr).into_owned();
        assert!(ended, "the blocked wait ran past its deadline");
        assert!(!output.status.success());
        assert!(said.contains("waited 1s for the probe"), "{said}");
    }
}
