//! A test run started paused for a debugger, as VS Code's "Debug Test": the
//! test host waits until a debugger attaches, and the run prints its process
//! id. The Apex CLI's `--debug` and VSTest's `VSTEST_HOST_DEBUG=1` both work
//! this way.

use std::num::NonZeroU32;
use std::time::Duration;

use bb_dotnet::ProjectPath;
use bb_process::OwnedGroup;
use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::process::{Child, ChildStdout, Command};

use super::{Call, DotnetBackend};
use crate::bb_engine::Selection;
use crate::model::Fqn;
use crate::parse::{parse_host_pid, HostPid};
use crate::TestkitError;

/// A debug run waiting for a debugger: the test host's process id. The
/// run and its host are stopped by [`DebugHost::stop`] (or on drop, the run).
#[derive(Debug)]
pub struct DebugHost {
    /// The test host to attach to (`coreclr`, `processId` as a string).
    pub pid: HostPid,
    child: Child,
    group: Option<OwnedGroup>,
}

impl DebugHost {
    /// Spawn `command` and wait up to `deadline` for the host id it prints.
    pub(crate) async fn start(
        mut command: Command,
        program: String,
        deadline: Duration,
    ) -> Result<DebugHost, TestkitError> {
        let mut child: Child = command
            .stdout(std::process::Stdio::piped())
            .spawn()
            .map_err(|source: std::io::Error| TestkitError::Spawn { program, source })?;
        let mut group: Option<OwnedGroup> = child
            .id()
            .and_then(NonZeroU32::new)
            .map(OwnedGroup::register);
        let stdout: Option<ChildStdout> = child.stdout.take();
        let found: Option<HostPid> = match stdout {
            Some(out) => tokio::time::timeout(deadline, first_pid(BufReader::new(out).lines()))
                .await
                .ok()
                .flatten(),
            None => None,
        };
        if let Some(pid) = found {
            return Ok(DebugHost { pid, child, group });
        }
        drop(group.take());
        let _reaped: Result<std::io::Result<std::process::ExitStatus>, _> =
            tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
        Err(TestkitError::NoHostPid(deadline))
    }

    /// Stop the run and everything it started, the waiting host included.
    pub async fn stop(mut self) {
        drop(self.group.take());
        let _reaped: Result<std::io::Result<std::process::ExitStatus>, _> =
            tokio::time::timeout(Duration::from_secs(2), self.child.wait()).await;
    }
}

async fn first_pid(mut lines: Lines<BufReader<ChildStdout>>) -> Option<HostPid> {
    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(pid) = parse_host_pid(&line) {
            return Some(pid);
        }
    }
    None
}

impl DotnetBackend {
    /// Start `specs` under `dotnet test` with `VSTEST_HOST_DEBUG=1` and wait
    /// for the test host id. The project must be built (`--no-build`).
    ///
    /// # Errors
    /// The run cannot start, or prints no id within `deadline`.
    pub async fn debug(
        &self,
        project: &ProjectPath,
        specs: &[Fqn],
        deadline: Duration,
    ) -> Result<DebugHost, TestkitError> {
        let selection: Selection = Selection {
            specs: specs.to_vec(),
            category: None,
        };
        let mut call: Call = Call::new(self.sdk.host(), self.workspace.root())
            .args(["test".into(), project.as_path().as_os_str().to_owned()])
            .args(["-c", self.configuration.as_str(), "--no-build", "--nologo"]);
        if let Some(filter) = super::dotnet::vstest_filter(&selection) {
            call = call.args(["--filter".to_owned(), filter]);
        }
        if let Some(settings) = &self.settings {
            call = call.args(["--settings".into(), settings.as_os_str().to_owned()]);
        }
        let mut command: Command = call.command();
        command.env("VSTEST_HOST_DEBUG", "1");
        DebugHost::start(command, call.name(), deadline).await
    }
}
