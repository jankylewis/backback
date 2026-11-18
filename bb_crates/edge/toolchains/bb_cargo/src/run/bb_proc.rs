//! One child process with a deadline, in its own process group: the only
//! way this crate runs a tool. A stop signal or the deadline ends the group.

use std::io::Read;
use std::num::NonZeroU32;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use bb_testkit::StopFlag;

use crate::CargoError;

/// What a finished process printed, and how it ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    /// The exit code; `None` when a signal ended it.
    pub code: Option<i32>,
    /// Everything on stdout.
    pub stdout: String,
    /// Everything on stderr.
    pub stderr: String,
    /// Wall time.
    pub elapsed: Duration,
}

/// Run `command` until it exits, `deadline` passes or `stop` is set.
///
/// # Errors
/// [`CargoError::Spawn`], [`CargoError::Timeout`], [`CargoError::Stopped`].
pub fn run(
    mut command: Command,
    what: &str,
    deadline: Duration,
    stop: Option<&StopFlag>,
) -> Result<Output, CargoError> {
    let started: Instant = Instant::now();
    let mut child: Child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()
        .map_err(|source: std::io::Error| CargoError::Spawn {
            what: what.to_owned(),
            source,
        })?;
    let group: Option<bb_process::OwnedGroup> =
        NonZeroU32::new(child.id()).map(bb_process::OwnedGroup::register);
    let out: std::thread::JoinHandle<String> = reader(child.stdout.take());
    let err: std::thread::JoinHandle<String> = reader(child.stderr.take());
    let status: Result<ExitStatus, CargoError> = wait(&mut child, what, (started, deadline), stop);
    // Ends whatever the tool left in its group, so both pipes close.
    drop(group);
    let stdout: String = out.join().unwrap_or_default();
    let stderr: String = err.join().unwrap_or_default();
    Ok(Output {
        code: status?.code(),
        stdout,
        stderr,
        elapsed: started.elapsed(),
    })
}

fn wait(
    child: &mut Child,
    what: &str,
    (started, deadline): (Instant, Duration),
    stop: Option<&StopFlag>,
) -> Result<ExitStatus, CargoError> {
    loop {
        let polled: Option<ExitStatus> =
            child
                .try_wait()
                .map_err(|source: std::io::Error| CargoError::Spawn {
                    what: what.to_owned(),
                    source,
                })?;
        if let Some(status) = polled {
            return Ok(status);
        }
        if stop.is_some_and(StopFlag::stopped) {
            let _killed: std::io::Result<()> = child.kill();
            return Err(CargoError::Stopped);
        }
        if started.elapsed() > deadline {
            let _killed: std::io::Result<()> = child.kill();
            return Err(CargoError::Timeout {
                what: what.to_owned(),
                after: deadline,
            });
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn reader<R: Read + Send + 'static>(pipe: Option<R>) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let mut text: String = String::new();
        if let Some(mut pipe) = pipe {
            let _read: std::io::Result<usize> = pipe.read_to_string(&mut text);
        }
        text
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tool_past_its_deadline_is_ended_with_a_typed_error() {
        let mut command: Command = Command::new("sleep");
        command.arg("5");
        let result: Result<Output, CargoError> =
            run(command, "sleep", Duration::from_millis(200), None);
        assert!(
            matches!(result, Err(CargoError::Timeout { .. })),
            "{result:?}"
        );
    }

    #[test]
    fn a_stopped_run_ends_with_stopped() {
        let stop: StopFlag = StopFlag::new();
        stop.stop();
        let mut command: Command = Command::new("sleep");
        command.arg("5");
        let result: Result<Output, CargoError> =
            run(command, "sleep", Duration::from_secs(10), Some(&stop));
        assert!(matches!(result, Err(CargoError::Stopped)), "{result:?}");
    }

    #[test]
    fn output_and_exit_code_come_back() {
        let mut command: Command = Command::new("sh");
        command.args(["-c", "echo out; echo err >&2; exit 3"]);
        let done: Output = run(command, "sh", Duration::from_secs(10), None).unwrap();
        assert_eq!(
            (done.code, done.stdout.as_str(), done.stderr.as_str()),
            (Some(3), "out\n", "err\n")
        );
    }
}
