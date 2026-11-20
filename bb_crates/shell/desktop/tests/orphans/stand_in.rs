//! Stand-in server groups for the launch sweep, and a record a dead
//! backback left (BBEOD42).

use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub const NEVER: &str = "Mon_Jan_1_00:00:00_2001";

/// A `sh` leader with a `sleep` member in a group of its own; its pgid.
pub fn group() -> u32 {
    let mut child: Child = Command::new("/bin/sh")
        .args(["-c", "/bin/sleep 300 & wait"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .unwrap();
    let pgid: u32 = child.id();
    let _reaper: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let _status: std::io::Result<std::process::ExitStatus> = child.wait();
    });
    std::thread::sleep(Duration::from_millis(100));
    pgid
}

pub fn token(pid: u32) -> String {
    let out: std::process::Output = Command::new("/bin/ps")
        .args(["-o", "lstart=", "-p", &pid.to_string()])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("_")
}

pub fn alive(pgid: u32) -> bool {
    Command::new("/bin/kill")
        .args(["-0", "--", &format!("-{pgid}")])
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status: std::process::ExitStatus| status.success())
}

pub fn gone_within(pgid: u32, within: Duration) -> bool {
    let started: Instant = Instant::now();
    while started.elapsed() < within {
        if !alive(pgid) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

pub fn kill(pgids: &[u32]) {
    for pgid in pgids {
        let _status: std::io::Result<std::process::ExitStatus> = Command::new("/bin/kill")
            .args(["-KILL", "--", &format!("-{pgid}")])
            .stderr(Stdio::null())
            .status();
    }
}

/// The record a `SIGKILL`ed backback (pid 99999998, long gone) left.
pub fn dead_record(children: &Path, groups: &[(u32, String)]) {
    std::fs::create_dir_all(children).unwrap();
    let lines: Vec<String> = groups
        .iter()
        .map(|(pgid, started): &(u32, String)| format!("group {pgid} {started}\n"))
        .collect();
    let text: String = format!(
        "backback children v1\nowner 99999998 Thu_Nov_19_22:00:00_2026\n{}",
        lines.concat()
    );
    std::fs::write(children.join("99999998"), text).unwrap();
}
