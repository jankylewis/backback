//! The Apex CLI in IDE mode, as the Apex Testkit extension drives it.

use std::path::{Path, PathBuf};
use std::time::Duration;

use bb_dotnet::{ProjectPath, Sdk};

use super::{assembly_of, build, shadow, Call, DebugHost, Finished, StopFlag, TempDir, Workspace};
use crate::bb_engine::{Deadlines, Selection, TestBackend};
use crate::model::{Fqn, SpecList};
use crate::parse::{parse_apex_list, ApexList, ApexSettings, CommandLine};
use crate::TestkitError;

#[path = "bb_hint.rs"]
mod hint;

/// `<command> test ...` with the workspace's `apexTestkit.*` settings.
#[derive(Debug, Clone)]
pub struct ApexBackend {
    settings: ApexSettings,
    workspace: Workspace,
    sdk: Sdk,
    deadlines: Deadlines,
    stop: Option<StopFlag>,
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
            stop: None,
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
        DebugHost::start(call.command(), call.name(), deadline).await
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

impl TestBackend for ApexBackend {
    fn with_stop(&self, stop: StopFlag) -> Self {
        let mut backend: Self = self.clone();
        backend.stop = Some(stop);
        backend
    }

    fn name(&self) -> &'static str {
        "apex"
    }

    async fn list(&self, project: &ProjectPath) -> Result<SpecList, TestkitError> {
        let original: PathBuf = assembly_of(
            &self.sdk,
            project,
            &self.settings.configuration,
            self.deadlines.list,
            self.stop.as_ref(),
        )
        .await?;
        let folder: TempDir = TempDir::new()?;
        let safe_assembly: PathBuf = shadow::assembly(&original, &folder)?;
        let stub: PathBuf = shadow::stub_project(project.as_path(), &safe_assembly, &folder)?;
        let mut call: Call =
            self.base(&ProjectPath::from_path(stub))
                .args(["--list", "--json", "--no-build"]);
        if let Some(flag) = &self.stop {
            call = call.stoppable(flag.clone());
        }
        let done: Finished = call
            .discover(self.deadlines.list)
            .await
            .map_err(hint::with_ide_hint)?;
        let list: ApexList = parse_apex_list(&done.stdout)?;
        if list.assembly != safe_assembly {
            return Err(TestkitError::Io {
                path: list.assembly,
                source: std::io::Error::from(std::io::ErrorKind::InvalidData),
            });
        }
        Ok(SpecList {
            project: list.project,
            assembly: original,
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
            self.stop.as_ref(),
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
        let mut call: Call = self.run_call(project, &selection.specs, trx);
        if let Some(stop) = &self.stop {
            call = call.stoppable(stop.clone());
        }
        call.finish(self.deadlines.run).await
    }
}
