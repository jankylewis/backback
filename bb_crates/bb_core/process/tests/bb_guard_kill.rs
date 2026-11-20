//! BBEOD42 (BB-PRC-009): a test process that owns a child group is
//! `SIGKILL`ed, as nextest kills a timed-out test. Its guard ends the group;
//! its record names only the groups still live.

#[cfg(test)]
mod probe {
    use std::io::{BufRead, BufReader};
    use std::os::unix::process::CommandExt;
    use std::path::Path;
    use std::process::{Child, Command, Stdio};
    use std::time::{Duration, Instant};

    pub const PROBE: &str = "BB_GUARD_KILL_PROBE";
    pub const NAME: &str = "an_owner_killed_by_sigkill_has_its_live_groups_ended_by_its_guard";

    pub fn group() -> Child {
        Command::new("/bin/sh")
            .args(["-c", "/bin/sleep 300 & wait"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()
            .unwrap()
    }

    pub fn alive(pid: u32, whole_group: bool) -> bool {
        let target: String = if whole_group {
            format!("-{pid}")
        } else {
            pid.to_string()
        };
        Command::new("/bin/kill")
            .args(["-0", "--", &target])
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success()
    }

    pub fn gone_within(pid: u32, whole_group: bool, within: Duration) -> bool {
        let started: Instant = Instant::now();
        while started.elapsed() < within {
            if !alive(pid, whole_group) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        false
    }

    pub fn started_probe(dir: &Path) -> (Child, Vec<u32>) {
        let mut child: Child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", NAME, "--test-threads=1", "--nocapture"])
            .env(PROBE, dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut line: String = String::new();
        let stdout: std::process::ChildStdout = child.stdout.take().unwrap();
        let mut reader: BufReader<std::process::ChildStdout> = BufReader::new(stdout);
        let said: String = loop {
            line.clear();
            assert!(
                reader.read_line(&mut line).unwrap() > 0,
                "the probe said nothing"
            );
            if let Some((_, after)) = line.split_once("bb_probe ") {
                break after.to_owned();
            }
        };
        let pids: Vec<u32> = said
            .split_whitespace()
            .map(|word: &str| word.parse().unwrap())
            .collect();
        (child, pids)
    }

    pub fn serve(dir: &std::ffi::OsStr) {
        let _swept: bb_process::Swept = bb_process::adopt(std::path::Path::new(dir)).unwrap();
        let kept: std::process::Child = group();
        let ended: std::process::Child = group();
        let kept_group: bb_process::OwnedGroup =
            bb_process::OwnedGroup::register(std::num::NonZeroU32::new(kept.id()).unwrap());
        drop(bb_process::OwnedGroup::register(
            std::num::NonZeroU32::new(ended.id()).unwrap(),
        ));
        let guard: u32 = bb_process::guard_pid().unwrap();
        println!("bb_probe {} {} {guard}", kept.id(), ended.id());
        for mut child in [kept, ended] {
            let _reaper: std::thread::JoinHandle<()> = std::thread::spawn(move || {
                let _status: std::io::Result<std::process::ExitStatus> = child.wait();
            });
        }
        std::thread::sleep(std::time::Duration::from_secs(300));
        drop(kept_group);
    }

    pub fn kill_and_check() {
        let dir: std::path::PathBuf =
            std::env::temp_dir().join(format!("bb-process-guardkill-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (mut owner, pids): (std::process::Child, Vec<u32>) = started_probe(&dir);
        let (kept, ended, guard): (u32, u32, u32) = (pids[0], pids[1], pids[2]);
        let record: String = std::fs::read_to_string(dir.join(owner.id().to_string())).unwrap();
        owner.kill().unwrap();
        owner.wait().unwrap();
        let kept_gone: bool = gone_within(kept, true, std::time::Duration::from_secs(3));
        let guard_gone: bool = gone_within(guard, false, std::time::Duration::from_secs(3));
        let _cleanup: std::io::Result<std::process::ExitStatus> =
            std::process::Command::new("/bin/kill")
                .args(["-KILL", "--", &format!("-{kept}"), &format!("-{ended}")])
                .stderr(std::process::Stdio::null())
                .status();
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(record.contains(&format!("group {kept} ")), "{record}");
        assert!(!record.contains(&format!("group {ended} ")), "{record}");
        assert!(
            kept_gone,
            "the guard ended the live group after the SIGKILL"
        );
        assert!(guard_gone, "the guard itself exits");
    }
}

#[test]
fn an_owner_killed_by_sigkill_has_its_live_groups_ended_by_its_guard() {
    match std::env::var_os(probe::PROBE) {
        Some(dir) => probe::serve(&dir),
        None => probe::kill_and_check(),
    }
}
