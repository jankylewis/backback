//! Who owns a temp folder: the pid in its name, if that process is alive
//! and started before the folder was last written. macOS reuses pids, so a
//! live pid alone can be another program that came later (BB-PRC-006).

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime};

/// Deadline for one `/bin/ps` call.
const PROBE_DEADLINE: Duration = Duration::from_millis(500);
/// Clock and `etime` rounding allowance: a process that started up to this
/// long after the folder's last write still counts as its owner.
const SLACK: Duration = Duration::from_secs(5);

/// The pid of `<prefix>-[<name>-...]<pid>[-<suffix>]`: the first part
/// after the prefix made only of digits.
pub(crate) fn owner(name: &str, prefix: &str) -> Option<u32> {
    let rest: &str = name.strip_prefix(prefix)?.strip_prefix('-')?;
    rest.split('-')
        .find(|part: &&str| !part.is_empty() && part.bytes().all(|b: u8| b.is_ascii_digit()))?
        .parse()
        .ok()
}

/// Whether `pid` still owns `folder`: alive (whoever runs it; `kill -0`
/// fails on another user's process) and not started after the folder's last
/// write. Any doubt (no probe, no time) says it owns it, so nothing is
/// removed on doubt.
pub(crate) fn owns(pid: u32, folder: &Path) -> bool {
    let Seen::Alive(elapsed): Seen = elapsed(pid) else {
        return false;
    };
    let written: Option<SystemTime> = std::fs::metadata(folder)
        .and_then(|meta: std::fs::Metadata| meta.modified())
        .ok();
    let (Some(elapsed), Some(written)): (Option<Duration>, Option<SystemTime>) = (elapsed, written)
    else {
        return true;
    };
    let Some(started): Option<SystemTime> = SystemTime::now().checked_sub(elapsed) else {
        return true;
    };
    written
        .checked_add(SLACK)
        .is_none_or(|until: SystemTime| started <= until)
}

/// What one `ps` probe saw of a pid.
enum Seen {
    /// No such process.
    Gone,
    /// Alive (or the probe failed) for this long, when known.
    Alive(Option<Duration>),
}

fn elapsed(pid: u32) -> Seen {
    let spawned: std::io::Result<std::process::Child> = Command::new("/bin/ps")
        .args(["-o", "etime=", "-p", &pid.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();
    let Ok(mut probe): std::io::Result<std::process::Child> = spawned else {
        return Seen::Alive(None);
    };
    let deadline: Instant = Instant::now() + PROBE_DEADLINE;
    loop {
        match probe.try_wait() {
            Ok(Some(status)) if status.success() => break,
            Ok(Some(_)) => return Seen::Gone,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(5)),
            Ok(None) | Err(_) => {
                let _stopped: std::io::Result<()> = probe.kill();
                let _reaped: std::io::Result<std::process::ExitStatus> = probe.wait();
                return Seen::Alive(None);
            }
        }
    }
    let mut out: String = String::new();
    if let Some(mut stdout) = probe.stdout.take() {
        let _read: std::io::Result<usize> = std::io::Read::read_to_string(&mut stdout, &mut out);
    }
    Seen::Alive(parse_etime(out.trim()))
}

/// `ps` elapsed time, `[[dd-]hh:]mm:ss`.
fn parse_etime(text: &str) -> Option<Duration> {
    let (days, clock): (u64, &str) = match text.split_once('-') {
        Some((d, rest)) => (d.parse().ok()?, rest),
        None => (0, text),
    };
    let mut secs: u64 = 0;
    for part in clock.split(':') {
        secs = secs.checked_mul(60)?.checked_add(part.parse().ok()?)?;
    }
    Some(Duration::from_secs(
        days.checked_mul(86_400)?.checked_add(secs)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_prefix_then_a_pid_names_an_owner() {
        assert_eq!(owner("bb-x-123", "bb-x"), Some(123));
        assert_eq!(owner("bb-x-123-ThreadId(2)", "bb-x"), Some(123));
        assert_eq!(owner("bb-x-e2e-123", "bb-x"), Some(123), "a fixture name");
        assert_eq!(owner("bb-x-e2e", "bb-x"), None, "no pid");
        assert_eq!(owner("bb-xy-123", "bb-x"), None);
        assert_eq!(owner("bb-x-", "bb-x"), None);
        assert_eq!(owner("bb-x-12a", "bb-x"), None);
    }

    #[test]
    fn ps_elapsed_times_parse() {
        assert_eq!(parse_etime("00:07"), Some(Duration::from_secs(7)));
        assert_eq!(parse_etime("12:34"), Some(Duration::from_secs(754)));
        assert_eq!(parse_etime("01:00:00"), Some(Duration::from_secs(3600)));
        assert_eq!(
            parse_etime("2-00:00:01"),
            Some(Duration::from_secs(172_801))
        );
        assert_eq!(parse_etime(""), None);
        assert_eq!(parse_etime("ab:cd"), None);
    }

    #[test]
    fn a_live_pid_that_started_after_the_folder_is_not_its_owner() {
        let dir: std::path::PathBuf =
            std::env::temp_dir().join(format!("bb-process-reuse-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let own: u32 = std::process::id();
        assert!(owns(own, &dir), "made by this run: owned");
        // A folder written before this test process started: the pid was
        // reused by this run, so the folder's real owner is gone.
        let old: SystemTime = SystemTime::now() - Duration::from_secs(86_400 * 400);
        std::fs::File::open(&dir)
            .unwrap()
            .set_modified(old)
            .unwrap();
        assert!(!owns(own, &dir), "older than its pid's process: not owned");
        assert!(!owns(99_999_999, &dir), "no such process");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
