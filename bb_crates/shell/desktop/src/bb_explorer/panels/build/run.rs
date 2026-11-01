//! Run `dotnet build` in its own process group (ended on quit), with a
//! deadline, and return everything it printed.
//!
//! `start` runs on the UI thread on purpose: on macOS a process spawned from
//! a background-QoS thread inherits the background state, and MSBuild's file
//! I/O is then throttled until the build looks hung (BB-PRC-002). Only the
//! waiting happens in the background.

use std::io::Read;
use std::num::NonZeroU32;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// A build that takes longer than this is stopped.
const DEADLINE: Duration = Duration::from_secs(600);

/// A started build: the child, its owned group and the output reader.
pub(super) struct Running {
    child: Child,
    owned: Option<bb_process::OwnedGroup>,
    reader: std::thread::JoinHandle<String>,
}

pub(super) fn start(target: &Path) -> Result<Running, String> {
    let mut child: Child = Command::new("dotnet")
        .arg("build")
        .arg(target)
        // No worker nodes or compiler server outlive the build: they would
        // hold the output pipe open.
        .args([
            "-nologo",
            "-v:minimal",
            "-nodeReuse:false",
            "-p:UseSharedCompilation=false",
        ])
        // A process group of its own must not hold the terminal as stdin.
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .map_err(|error: std::io::Error| format!("dotnet: {error}"))?;
    let owned: Option<bb_process::OwnedGroup> =
        NonZeroU32::new(child.id()).map(bb_process::OwnedGroup::register);
    let mut stdout: std::process::ChildStdout = child.stdout.take().ok_or("no output pipe")?;
    let reader: std::thread::JoinHandle<String> = std::thread::spawn(move || {
        let mut text: String = String::new();
        let _read: std::io::Result<usize> = stdout.read_to_string(&mut text);
        text
    });
    Ok(Running {
        child,
        owned,
        reader,
    })
}

/// Wait for the build (at most [`DEADLINE`]) and return its output.
pub(super) fn wait(running: Running) -> Result<String, String> {
    let Running {
        mut child,
        owned,
        reader,
    }: Running = running;
    let started: Instant = Instant::now();
    loop {
        if child
            .try_wait()
            .map_err(|error: std::io::Error| error.to_string())?
            .is_some()
        {
            break;
        }
        if started.elapsed() > DEADLINE {
            let _killed: std::io::Result<()> = child.kill();
            return Err(format!("stopped after {} s", DEADLINE.as_secs()));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    // End whatever the build left in its group, so the pipe closes and the
    // reader returns.
    drop(owned);
    reader
        .join()
        .map_err(|_: Box<dyn std::any::Any + Send>| "output reader failed".to_owned())
}
