//! The Apex CLI in IDE mode, as the Apex Testkit extension drives it.

use std::path::Path;
use std::time::Duration;

use bb_dotnet::{ProjectPath, Sdk};
use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::process::{Child, ChildStdout};

use super::tree::stop_tree;
use super::{build, Call, Finished, Workspace};
use crate::bb_engine::{Deadlines, Selection, TestBackend};
use crate::model::{Fqn, SpecList};
use crate::parse::{parse_apex_list, parse_host_pid, ApexList, ApexSettings, CommandLine, HostPid};
use crate::TestkitError;

/// `<command> test ...` with the workspace's `apexTestkit.*` settings.
#[derive(Debug, Clone)]
pub struct ApexBackend {
    settings: ApexSettings,
    workspace: Workspace,
    sdk: Sdk,
    deadlines: Deadlines,
}

/// A debug run waiting for a debugger: the test host's process id. The
/// run and its host are stopped by [`DebugHost::stop`] (or on drop, the run).
#[derive(Debug)]
pub struct DebugHost {
    /// The test host to attach to (`coreclr`, `processId` as a string).
    pub pid: HostPid,
    child: Child,
}

impl ApexBackend {
    /// A backend for the workspace, with its settings.
    ///
    /// # Errors
    /// No .NET SDK (needed for `buildBeforeRun`).
    pub fn new(workspace: Workspace, settings: ApexSettings) -> Result<Self, TestkitError> {
        Ok(Self {
            settings,
            workspace,
            sdk: Sdk::locate()?,
            deadlines: Deadlines::default(),
        })
    }

    /// The same backend with another command prefix.
    #[must_use]
    pub fn with_command(mut self, command: CommandLine) -> Self {
        self.settings.command = command;
        self
    }

    /// The same backend with other deadlines.
    #[must_use]
    pub fn with_deadlines(mut self, deadlines: Deadlines) -> Self {
        self.deadlines = deadlines;
        self
    }

    /// The settings in use.
    #[must_use]
    pub fn settings(&self) -> &ApexSettings {
        &self.settings
    }

    /// Start `specs` under `--debug` and wait for the test host id.
    ///
    /// # Errors
    /// The run cannot start, or prints no id within `deadline`.
    pub async fn debug(
        &self,
        project: &ProjectPath,
        specs: &[Fqn],
        trx: &Path,
        deadline: Duration,
    ) -> Result<DebugHost, TestkitError> {
        let call: Call = self.run_call(project, specs, trx).args(["--debug"]);
        let mut child: Child = call
            .command()
            .stdout(std::process::Stdio::piped())
            .spawn()
            .map_err(|source: std::io::Error| TestkitError::Spawn {
                program: call.name(),
                source,
            })?;
        let stdout: Option<ChildStdout> = child.stdout.take();
        let found: Option<HostPid> = match stdout {
            Some(out) => tokio::time::timeout(deadline, first_pid(BufReader::new(out).lines()))
                .await
                .ok()
                .flatten(),
            None => None,
        };
        if let Some(pid) = found {
            return Ok(DebugHost { pid, child });
        }
        stop_tree(&mut child).await;
        Err(TestkitError::NoHostPid(deadline))
    }

    fn base(&self, project: &ProjectPath) -> Call {
        let command: &CommandLine = &self.settings.command;
        Call::new(&command.program, self.workspace.root())
            .args(command.prefix.iter().map(String::as_str))
            .args([
                "test".into(),
                "--project".into(),
                project.as_path().as_os_str().to_owned(),
            ])
            .args(["-c", self.settings.configuration.as_str()])
    }

    fn run_call(&self, project: &ProjectPath, specs: &[Fqn], trx: &Path) -> Call {
        let mut call: Call = self.base(project);
        if !specs.is_empty() {
            let joined: String = specs
                .iter()
                .map(Fqn::as_str)
                .collect::<Vec<&str>>()
                .join(",");
            call = call.args(["--specs".to_owned(), joined]);
        }
        if let Some(settings) = &self.settings.run_settings {
            call = call.args(["--settings".into(), settings.as_os_str().to_owned()]);
        }
        call.args([
            "--trx".into(),
            trx.as_os_str().to_owned(),
            "--no-build".into(),
        ])
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

impl DebugHost {
    /// Stop the run and everything it started, the waiting host included.
    pub async fn stop(mut self) {
        stop_tree(&mut self.child).await;
    }
}

impl TestBackend for ApexBackend {
    fn name(&self) -> &'static str {
        "apex"
    }

    async fn list(&self, project: &ProjectPath) -> Result<SpecList, TestkitError> {
        let done: Finished = self
            .base(project)
            .args(["--list", "--json", "--no-build"])
            .discover(self.deadlines.list)
            .await?;
        let list: ApexList = parse_apex_list(&done.stdout)?;
        Ok(SpecList {
            project: list.project,
            assembly: list.assembly,
            specs: list.specs,
            elapsed: done.elapsed,
        })
    }

    fn builds_before_run(&self) -> bool {
        self.settings.build_before_run
    }

    async fn build(&self, project: &ProjectPath) -> Result<(), TestkitError> {
        build(
            &self.sdk,
            project,
            &self.settings.configuration,
            self.workspace.root(),
            self.deadlines.build,
        )
        .await
    }

    async fn execute(
        &self,
        project: &ProjectPath,
        selection: &Selection,
        trx: &Path,
    ) -> Result<Finished, TestkitError> {
        if selection.category.is_some() {
            return Err(TestkitError::Unsupported {
                backend: "apex",
                what: "filter by category (the Apex CLI selects by --specs or --topic)",
            });
        }
        self.run_call(project, &selection.specs, trx)
            .finish(self.deadlines.run)
            .await
    }
}
