//! Plain `dotnet` for NUnit, xUnit and `MSTest` projects: VSTest or
//! Microsoft.Testing.Platform, read from the project file.

mod bb_vstest;
mod mtp;
pub(super) mod shadow;

use std::path::{Path, PathBuf};

use bb_dotnet::testing::TestPlatform;
use bb_dotnet::{Capability, ProjectInfo, ProjectPath, Sdk};

use super::{assembly_of, build, Finished, StopFlag, Workspace};
use crate::bb_engine::{Deadlines, Selection, TestBackend};
use crate::model::{Configuration, SpecList};
use crate::TestkitError;

/// Plain `dotnet` for NUnit, xUnit and `MSTest`: VSTest or MTP by project.
#[derive(Debug, Clone)]
pub struct DotnetBackend {
    pub(crate) sdk: Sdk,
    pub(crate) workspace: Workspace,
    pub(crate) configuration: Configuration,
    pub(crate) platform: TestPlatform,
    pub(crate) xunit: bool,
    pub(crate) settings: Option<PathBuf>,
    pub(crate) deadlines: Deadlines,
    pub(crate) build_first: bool,
    pub(crate) stop: Option<StopFlag>,
}

impl DotnetBackend {
    /// A backend for `project`, its platform read from the project file.
    /// Runs use the project's own runsettings unless `settings` is given.
    ///
    /// # Errors
    /// No .NET SDK, or the project file cannot be read.
    pub fn for_project(
        project: &ProjectPath,
        configuration: Configuration,
        settings: Option<PathBuf>,
        build_first: bool,
    ) -> Result<Self, TestkitError> {
        let info: ProjectInfo = ProjectInfo::load(project)?;
        Ok(Self {
            sdk: Sdk::locate()?,
            workspace: Workspace::of(project),
            configuration,
            platform: TestPlatform::of(&info),
            xunit: info.has(Capability::XUnit),
            settings,
            deadlines: Deadlines::default(),
            build_first,
            stop: None,
        })
    }

    /// The same backend with other deadlines.
    #[must_use]
    pub fn with_deadlines(mut self, deadlines: Deadlines) -> Self {
        self.deadlines = deadlines;
        self
    }
}

impl TestBackend for DotnetBackend {
    fn with_stop(&self, stop: StopFlag) -> Self {
        let mut backend: Self = self.clone();
        backend.stop = Some(stop);
        backend
    }
    fn name(&self) -> &'static str {
        match self.platform {
            TestPlatform::VsTest => "vstest",
            TestPlatform::Mtp => "mtp",
        }
    }

    async fn list(&self, project: &ProjectPath) -> Result<SpecList, TestkitError> {
        let assembly: PathBuf = assembly_of(
            &self.sdk,
            project,
            &self.configuration,
            self.deadlines.list,
            self.stop.as_ref(),
        )
        .await?;
        match self.platform {
            TestPlatform::VsTest => bb_vstest::list(self, project, assembly).await,
            TestPlatform::Mtp => mtp::list(self, project, assembly).await,
        }
    }

    fn builds_before_run(&self) -> bool {
        self.build_first
    }

    async fn build(&self, project: &ProjectPath) -> Result<(), TestkitError> {
        build(
            &self.sdk,
            project,
            &self.configuration,
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
        match self.platform {
            TestPlatform::VsTest => bb_vstest::execute(self, project, selection, trx).await,
            TestPlatform::Mtp => {
                let assembly: PathBuf = assembly_of(
                    &self.sdk,
                    project,
                    &self.configuration,
                    self.deadlines.list,
                    self.stop.as_ref(),
                )
                .await?;
                mtp::execute(self, &assembly, selection, trx).await
            }
        }
    }
}
