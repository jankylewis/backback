use std::path::PathBuf;
use std::time::Duration;
#[path = "bb_backend.rs"]
mod backend;
mod runner;
mod scan;
pub(crate) use runner::{run_category, run_class, run_failed, run_project, run_spec, stop};
pub(crate) use scan::{find, scan};

use bb_dotnet::ProjectPath;
use bb_testkit::{
    ApexBackend, ApexSettings, Configuration, DotnetBackend, SpecList, TestBackend, Workspace,
};
use gpui::{AsyncWindowContext, Context, WeakEntity, Window};

use crate::ExplorerView;

type ScanResult = Result<(Vec<PathBuf>, Vec<PathBuf>), String>;
type ListResult = Result<SpecList, String>;

/// Each `dotnet` call of a build before a list (BBEOD20).
const BUILD: Duration = Duration::from_secs(180);

/// Why a list did not come back.
enum ListError {
    /// The test assembly is missing: `--no-build` cannot list it.
    NotBuilt(PathBuf),
    Other(String),
}

impl ListError {
    fn of(error: bb_testkit::TestkitError) -> Self {
        if let bb_testkit::TestkitError::NotBuilt(assembly) = error {
            Self::NotBuilt(assembly)
        } else {
            Self::Other(error.to_string())
        }
    }
}

pub(crate) fn open(view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>) {
    view.tests.side = super::Side::Tests;
    if view.tests.scan != super::Scan::NotStarted {
        cx.notify();
        return;
    }
    view.tests.scan = super::Scan::Running;
    let root: PathBuf = view.root.clone();
    let job: gpui::Task<ScanResult> = cx.background_executor().spawn(async move { scan(&root) });
    cx.spawn_in(
        window,
        async move |view: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext| {
            let result: Result<(Vec<PathBuf>, Vec<PathBuf>), String> = job.await;
            view.update_in(
                cx,
                |view: &mut ExplorerView, _: &mut Window, cx: &mut Context<ExplorerView>| {
                    view.tests.scan = super::Scan::NotStarted;
                    match result {
                        Ok((projects, settings)) => {
                            view.tests.scan = super::Scan::Done;
                            for project in projects {
                                view.tests.model.add_project(project);
                            }
                            view.tests.settings_choices = settings;
                        }
                        Err(error) => view.tests.message = Some(error),
                    }
                    cx.notify();
                },
            )
            .ok();
        },
    )
    .detach();
    cx.notify();
}

pub(crate) fn toggle_project(
    view: &mut ExplorerView,
    path: PathBuf,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let project: &mut super::model::ProjectTests = match view.tests.model.project_mut(&path) {
        Some(project) => project,
        None => return,
    };
    project.expanded = !project.expanded;
    if !project.expanded || !project.specs.is_empty() || project.loading {
        cx.notify();
        return;
    }
    start_list(view, path, window, cx);
}

/// List `path`'s tests in the background; when the list ends, the filter's
/// search lists its next project.
fn start_list(
    view: &mut ExplorerView,
    path: PathBuf,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    if let Some(project) = view.tests.model.project_mut(&path) {
        project.loading = true;
    }
    let root: PathBuf = view.root.clone();
    let request: PathBuf = path.clone();
    let may_build: bool = !view.tabs.no_write();
    #[cfg(test)]
    let job: gpui::Task<ListResult> = cx
        .background_executor()
        .spawn(async move { list(root, &request, may_build) });
    #[cfg(not(test))]
    let receive: tokio::sync::oneshot::Receiver<ListResult> =
        list_in_thread(root, request, may_build);
    cx.spawn_in(
        window,
        async move |view: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext| {
            #[cfg(test)]
            let result: ListResult = job.await;
            #[cfg(not(test))]
            let result: ListResult =
                receive
                    .await
                    .unwrap_or_else(|error: tokio::sync::oneshot::error::RecvError| {
                        Err(error.to_string())
                    });
            view.update_in(
                cx,
                |view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>| {
                    apply_list(view, &path, result, cx);
                    find::list_next(view, window, cx);
                },
            )
            .ok();
        },
    )
    .detach();
    cx.notify();
}

fn apply_list(
    view: &mut ExplorerView,
    path: &std::path::Path,
    result: ListResult,
    cx: &mut Context<ExplorerView>,
) {
    match result {
        Ok(list) => view.tests.model.merge(path, list),
        Err(error) => {
            if let Some(project) = view.tests.model.project_mut(path) {
                project.loading = false;
                project.error = Some(error);
            }
        }
    }
    cx.notify();
}

#[cfg(not(test))]
fn list_in_thread(
    root: PathBuf,
    request: PathBuf,
    may_build: bool,
) -> tokio::sync::oneshot::Receiver<ListResult> {
    let (send, receive): (
        tokio::sync::oneshot::Sender<ListResult>,
        tokio::sync::oneshot::Receiver<ListResult>,
    ) = tokio::sync::oneshot::channel();
    let _worker: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let result: ListResult = list(root, &request, may_build);
        let _delivered: Result<(), ListResult> = send.send(result);
    });
    receive
}

fn runtime() -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error: std::io::Error| error.to_string())
}

/// List the project's tests. A never-built project is built once first
/// (BBEOD20, as VS Code and Rider do), unless `may_build` is false:
/// `--no-write` keeps the solution unchanged, and a build writes bin/ and
/// obj/.
fn list(root: PathBuf, path: &std::path::Path, may_build: bool) -> ListResult {
    let rt: tokio::runtime::Runtime = runtime()?;
    match list_once(&rt, root.clone(), path) {
        Ok(list) => Ok(list),
        Err(ListError::NotBuilt(_)) if may_build => {
            rt.block_on(bb_dap::target::resolve(path, BUILD))
                .map_err(|error: bb_dap::target::TargetError| error.to_string())?;
            list_once(&rt, root, path).map_err(|error: ListError| match error {
                ListError::NotBuilt(assembly) => {
                    format!("not built after a build: {}", assembly.display())
                }
                ListError::Other(text) => text,
            })
        }
        Err(ListError::NotBuilt(assembly)) => Err(format!(
            "not built: {} (--no-write: build the project outside backback)",
            assembly.display()
        )),
        Err(ListError::Other(text)) => Err(text),
    }
}

fn list_once(
    rt: &tokio::runtime::Runtime,
    root: PathBuf,
    path: &std::path::Path,
) -> Result<SpecList, ListError> {
    let project: ProjectPath = ProjectPath::from_path(path.to_path_buf());
    if super::uses_apex(&root, path) {
        let settings: ApexSettings = super::apex_settings(&root).map_err(ListError::Other)?;
        let backend: ApexBackend =
            ApexBackend::new(Workspace::at(root), settings).map_err(ListError::of)?;
        rt.block_on(backend.list(&project)).map_err(ListError::of)
    } else {
        let backend: DotnetBackend =
            DotnetBackend::for_project(&project, Configuration::debug(), None, false)
                .map_err(ListError::of)?;
        rt.block_on(backend.list(&project)).map_err(ListError::of)
    }
}
