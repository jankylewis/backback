//! `dotnet` that leaves no build server behind (BBEOD43). Every `dotnet`
//! backback or its tests start comes from [`command`]; a test-side build
//! runs through [`run`]. A guard test (`tests/no_servers.rs`) refuses any
//! other `Command::new("dotnet")` in the workspace.
//!
//! A plain `dotnet build` hands its compile to the per-user compiler server
//! (`VBCSCompiler`), starting one when none runs, and MSBuild may keep worker
//! nodes for reuse. Either outlives the build: BBEOD42 found a `VBCSCompiler`
//! with parent 1 left by a test fixture's build.

use std::os::unix::process::CommandExt;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

/// Read by every MSBuild a call starts, nested ones too (`dotnet test`
/// building first, the Apex CLI): no node kept for reuse, and every compile
/// in the build's own process, never in a compiler server.
pub const ENV: [(&str, &str); 2] = [
    ("MSBUILDDISABLENODEREUSE", "1"),
    ("UseSharedCompilation", "false"),
];

/// The same as `dotnet build` / `dotnet msbuild` switches (Build Solution
/// and the debugger's build show them in their command line).
pub const FLAGS: [&str; 2] = ["-nodeReuse:false", "-p:UseSharedCompilation=false"];

/// Time the group gets after TERM before KILL.
const GRACE: Duration = Duration::from_millis(500);
const POLL: Duration = Duration::from_millis(20);

/// What [`run`] left: the output, and the pid of the `dotnet` it started
/// (the leader of the group [`run`] ended).
#[derive(Debug)]
pub struct Ran {
    /// The captured output and exit status.
    pub output: Output,
    /// The pid of the `dotnet` process, also its process group.
    pub pid: u32,
}

/// `dotnet`: the leader of its own process group, with [`ENV`].
#[must_use]
pub fn command() -> Command {
    let mut command: Command = Command::new("dotnet");
    command.envs(ENV).stdin(Stdio::null()).process_group(0);
    command
}

/// Run `command` (from [`command`]) to its end with its output captured,
/// then end whatever its group still holds.
///
/// # Errors
/// The process cannot start or be waited for.
pub fn run(command: &mut Command) -> std::io::Result<Ran> {
    let child: std::process::Child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let pid: u32 = child.id();
    let waited: std::io::Result<Output> = child.wait_with_output();
    end_group(pid);
    Ok(Ran {
        output: waited?,
        pid,
    })
}

/// End every process still in the group `leader` leads: TERM, a short
/// grace, then KILL. Only that group: never a server the user started.
pub fn end_group(leader: u32) {
    let group: String = format!("-{leader}");
    if !signal("-TERM", &group) {
        return;
    }
    let deadline: Instant = Instant::now() + GRACE;
    while Instant::now() < deadline {
        if !signal("-0", &group) {
            return;
        }
        std::thread::sleep(POLL);
    }
    let _killed: bool = signal("-KILL", &group);
}

/// `kill <signal> -- <group>`; whether any member got it.
fn signal(signal: &str, group: &str) -> bool {
    Command::new("/bin/kill")
        .args([signal, "--", group])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status: std::process::ExitStatus| status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_ends_what_the_group_left_behind() {
        let mut command: Command = Command::new("/bin/sh");
        command
            .args([
                "-c",
                "(trap '' TERM; exec /bin/sleep 600) >/dev/null 2>&1 & echo started",
            ])
            .process_group(0);
        let ran: Ran = run(&mut command).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&ran.output.stdout).trim(),
            "started"
        );
        let group: String = format!("-{}", ran.pid);
        let left: bool = signal("-0", &group);
        if left {
            let _ended: bool = signal("-KILL", &group);
        }
        assert!(!left, "a sleep 600 that ignores TERM outlived run");
    }
}
