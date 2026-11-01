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
    if super::super::uses_apex(path) {
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
