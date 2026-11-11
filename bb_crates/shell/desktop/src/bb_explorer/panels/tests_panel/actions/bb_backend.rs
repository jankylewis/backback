use bb_dotnet::ProjectPath;
use bb_testkit::{
    run_with_stop, ApexBackend, ApexSettings, Configuration, DotnetBackend, RunReport, Selection,
    StopFlag, Workspace,
};
use std::path::{Path, PathBuf};

pub(super) type Reports = Vec<(PathBuf, Result<RunReport, String>)>;

#[cfg(not(test))]
pub(super) fn run_in_thread(
    root: PathBuf,
    settings: Option<PathBuf>,
    requests: Vec<(PathBuf, Selection)>,
    stop: StopFlag,
) -> tokio::sync::oneshot::Receiver<Reports> {
    let (send, receive): (
        tokio::sync::oneshot::Sender<Reports>,
        tokio::sync::oneshot::Receiver<Reports>,
    ) = tokio::sync::oneshot::channel();
    let _worker: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let reports: Reports = run_reports(&root, settings.as_deref(), requests, &stop);
        let _delivered: Result<(), Reports> = send.send(reports);
    });
    receive
}

pub(super) fn run_reports(
    root: &Path,
    settings: Option<&Path>,
    requests: Vec<(PathBuf, Selection)>,
    stop: &StopFlag,
) -> Reports {
    let mut reports: Reports = Vec::new();
    for (path, selection) in requests {
        if stop.stopped() {
            break;
        }
        let report: Result<RunReport, String> = execute(
            root.to_path_buf(),
            &path,
            &selection,
            settings.map(Path::to_path_buf),
            stop.clone(),
        );
        reports.push((path, report));
    }
    reports
}

pub(super) fn execute(
    root: PathBuf,
    path: &Path,
    selection: &Selection,
    settings_override: Option<PathBuf>,
    stop: StopFlag,
) -> Result<RunReport, String> {
    let project: ProjectPath = ProjectPath::from_path(path.to_path_buf());
    let rt: tokio::runtime::Runtime = super::runtime()?;
    if super::super::uses_apex(&root, path) {
        let mut settings: ApexSettings = super::super::apex_settings(&root)?;
        if let Some(chosen) = settings_override {
            settings.run_settings = Some(chosen);
        }
        let backend: ApexBackend = ApexBackend::new(Workspace::at(root), settings)
            .map_err(|error: bb_testkit::TestkitError| error.to_string())?;
        rt.block_on(run_with_stop(&backend, &project, selection, stop))
            .map_err(|error: bb_testkit::TestkitError| error.to_string())
    } else {
        let backend: DotnetBackend =
            DotnetBackend::for_project(&project, Configuration::debug(), settings_override, true)
                .map_err(|error: bb_testkit::TestkitError| error.to_string())?;
        rt.block_on(run_with_stop(&backend, &project, selection, stop))
            .map_err(|error: bb_testkit::TestkitError| error.to_string())
    }
}

/// How long a paused test run may take to name its test host.
const HOST_DEADLINE: std::time::Duration = std::time::Duration::from_secs(120);

/// A test started paused for the debugger (Debug on a test): built first,
/// then run by the backend that runs it (Apex CLI `--debug`, or `dotnet
/// test` with `VSTEST_HOST_DEBUG=1`). The folder holds the Apex run's TRX
/// and must live as long as the host.
pub(crate) async fn debug_host(
    root: PathBuf,
    path: &Path,
    fqn: &bb_testkit::Fqn,
) -> Result<(bb_testkit::DebugHost, bb_process::OwnedTempDir), String> {
    use bb_testkit::TestBackend;
    let project: ProjectPath = ProjectPath::from_path(path.to_path_buf());
    let specs: [bb_testkit::Fqn; 1] = [fqn.clone()];
    let folder: bb_process::OwnedTempDir = bb_process::OwnedTempDir::create_with("bb-run", "debug")
        .map_err(|error: std::io::Error| error.to_string())?;
    let host: bb_testkit::DebugHost = if super::super::uses_apex(&root, path) {
        let settings: ApexSettings = super::super::apex_settings(&root)?;
        let backend: ApexBackend = ApexBackend::new(Workspace::at(root), settings)
            .map_err(|error: bb_testkit::TestkitError| error.to_string())?;
        if backend.builds_before_run() {
            backend
                .build(&project)
                .await
                .map_err(|error: bb_testkit::TestkitError| error.to_string())?;
        }
        let trx: PathBuf = folder.path().join("debug.trx");
        backend
            .debug(&project, &specs, &trx, HOST_DEADLINE)
            .await
            .map_err(|error: bb_testkit::TestkitError| error.to_string())?
    } else {
        let backend: DotnetBackend =
            DotnetBackend::for_project(&project, Configuration::debug(), None, true)
                .map_err(|error: bb_testkit::TestkitError| error.to_string())?;
        backend
            .build(&project)
            .await
            .map_err(|error: bb_testkit::TestkitError| error.to_string())?;
        backend
            .debug(&project, &specs, HOST_DEADLINE)
            .await
            .map_err(|error: bb_testkit::TestkitError| error.to_string())?
    };
    Ok((host, folder))
}
