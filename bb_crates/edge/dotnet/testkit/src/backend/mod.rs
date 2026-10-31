//! The two backends, the workspace folder, and what both need: the test
//! assembly path, `dotnet build`, a TRX folder that removes itself.

mod apex;
mod bb_proc;
mod dotnet;
mod tree;

pub use apex::{ApexBackend, DebugHost};
pub use bb_proc::{Call, Finished};
pub use dotnet::DotnetBackend;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use bb_dotnet::{ProjectPath, Sdk};

use crate::model::Configuration;
use crate::parse::SETTINGS_FILE;
use crate::TestkitError;

/// The folder the tools run in: the nearest folder above the project with
/// `.vscode/settings.json`, else with a solution file, else the project's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace(PathBuf);

impl Workspace {
    /// The workspace folder of `project`.
    #[must_use]
    pub fn of(project: &ProjectPath) -> Self {
        let dir: &Path = project.as_path().parent().unwrap_or_else(|| Path::new("."));
        let found: Option<&Path> = dir
            .ancestors()
            .find(|d: &&Path| d.join(SETTINGS_FILE).is_file())
            .or_else(|| dir.ancestors().find(|d: &&Path| has_solution(d)));
        Self(found.unwrap_or(dir).to_path_buf())
    }

    /// A given folder.
    #[must_use]
    pub fn at(root: PathBuf) -> Self {
        Self(root)
    }

    /// The folder.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.0
    }
}

fn has_solution(dir: &Path) -> bool {
    std::fs::read_dir(dir).is_ok_and(|entries: std::fs::ReadDir| {
        entries.flatten().any(|e: std::fs::DirEntry| {
            e.path()
                .extension()
                .is_some_and(|x: &std::ffi::OsStr| x == "sln" || x == "slnx")
        })
    })
}

/// A fresh temporary folder, removed when dropped.
#[derive(Debug)]
pub struct TempDir(PathBuf);

impl TempDir {
    /// Create one under the system temp folder.
    ///
    /// # Errors
    /// The folder cannot be created.
    pub fn new() -> Result<Self, TestkitError> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let n: u64 = NEXT.fetch_add(1, Ordering::Relaxed);
        let path: PathBuf =
            std::env::temp_dir().join(format!("bb-testkit-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&path).map_err(|source: std::io::Error| TestkitError::Io {
            path: path.clone(),
            source,
        })?;
        Ok(Self(path))
    }

    /// The folder.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // Best effort: a TRX folder left behind is only temp space.
        let _removed: std::io::Result<()> = std::fs::remove_dir_all(&self.0);
    }
}

/// `TargetPath` of `project` in `configuration`; it must exist (no build).
pub(crate) async fn assembly_of(
    sdk: &Sdk,
    project: &ProjectPath,
    configuration: &Configuration,
    deadline: Duration,
) -> Result<PathBuf, TestkitError> {
    let dir: &Path = project.as_path().parent().unwrap_or_else(|| Path::new("."));
    let done: Finished = Call::new(sdk.host(), dir)
        .args(["msbuild".into(), project.as_path().as_os_str().to_owned()])
        .args([
            "-getProperty:TargetPath".to_owned(),
            format!("-p:Configuration={}", configuration.as_str()),
            "-nologo".to_owned(),
        ])
        .succeed(deadline)
        .await?;
    let path: PathBuf = PathBuf::from(done.stdout.trim());
    if path.is_file() {
        Ok(path)
    } else {
        Err(TestkitError::NotBuilt(path))
    }
}

/// `dotnet build <project> -c <configuration>`.
pub(crate) async fn build(
    sdk: &Sdk,
    project: &ProjectPath,
    configuration: &Configuration,
    cwd: &Path,
    deadline: Duration,
) -> Result<(), TestkitError> {
    Call::new(sdk.host(), cwd)
        .args(["build".into(), project.as_path().as_os_str().to_owned()])
        .args(["-c", configuration.as_str(), "-nologo", "-v", "q"])
        .succeed(deadline)
        .await
        .map(|_: Finished| ())
}

/// The project's file stem, as the Apex list names projects.
pub(crate) fn project_name(project: &ProjectPath) -> String {
    project
        .as_path()
        .file_stem()
        .map(|s: &std::ffi::OsStr| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}
