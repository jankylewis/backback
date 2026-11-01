use std::path::PathBuf;
#[path = "bb_backend.rs"]
mod backend;
mod runner;
mod scan;
pub(crate) use runner::{run_category, run_class, run_failed, run_project, run_spec, stop};
pub(crate) use scan::scan;

use bb_dotnet::ProjectPath;
use bb_testkit::{
    ApexBackend, ApexSettings, Configuration, DotnetBackend, SpecList, TestBackend, Workspace,
};
use gpui::{AsyncWindowContext, Context, WeakEntity, Window};

use crate::ExplorerView;

type ScanResult = Result<(Vec<PathBuf>, Vec<PathBuf>), String>;
type ListResult = Result<SpecList, String>;

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
    project.loading = true;
    let root: PathBuf = view.root.clone();
    let request: PathBuf = path.clone();
    #[cfg(test)]
    let job: gpui::Task<ListResult> = cx
        .background_executor()
        .spawn(async move { list(root, &request) });
    #[cfg(not(test))]
    let receive: tokio::sync::oneshot::Receiver<ListResult> = list_in_thread(root, request);
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
                |view: &mut ExplorerView, _: &mut Window, cx: &mut Context<ExplorerView>| {
                    apply_list(view, &path, result, cx);
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
fn list_in_thread(root: PathBuf, request: PathBuf) -> tokio::sync::oneshot::Receiver<ListResult> {
    let (send, receive): (
        tokio::sync::oneshot::Sender<ListResult>,
        tokio::sync::oneshot::Receiver<ListResult>,
    ) = tokio::sync::oneshot::channel();
    let _worker: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let result: ListResult = list(root, &request);
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

fn list(root: PathBuf, path: &std::path::Path) -> Result<SpecList, String> {
    let project: ProjectPath = ProjectPath::from_path(path.to_path_buf());
    let rt: tokio::runtime::Runtime = runtime()?;
    if super::uses_apex(path) {
        let settings: ApexSettings = super::apex_settings(&root)?;
        let backend: ApexBackend = ApexBackend::new(Workspace::at(root), settings)
            .map_err(|error: bb_testkit::TestkitError| error.to_string())?;
        rt.block_on(backend.list(&project))
            .map_err(|error: bb_testkit::TestkitError| error.to_string())
    } else {
        let backend: DotnetBackend =
            DotnetBackend::for_project(&project, Configuration::debug(), None, false)
                .map_err(|error: bb_testkit::TestkitError| error.to_string())?;
        rt.block_on(backend.list(&project))
            .map_err(|error: bb_testkit::TestkitError| error.to_string())
    }
}
