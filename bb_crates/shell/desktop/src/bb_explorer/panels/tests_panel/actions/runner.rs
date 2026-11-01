use super::super::model::ProjectTests;
#[cfg(not(test))]
use super::backend::run_in_thread;
#[cfg(test)]
use super::backend::run_reports;
use super::backend::Reports;
use crate::ExplorerView;
use bb_testkit::{Category, Fqn, FqnParts, Leaf, RunReport, Selection, StopFlag};
use gpui::{AsyncWindowContext, Context, WeakEntity, Window};
use std::fmt::Write;
use std::path::{Path, PathBuf};
pub(crate) fn run_spec(
    view: &mut ExplorerView,
    path: PathBuf,
    fqn: Fqn,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    run_one(
        view,
        path,
        Selection {
            specs: vec![fqn],
            category: None,
        },
        window,
        cx,
    );
}
pub(crate) fn run_project(
    view: &mut ExplorerView,
    path: PathBuf,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    run_one(view, path, Selection::default(), window, cx);
}
pub(crate) fn run_class(
    view: &mut ExplorerView,
    path: PathBuf,
    owner: &(String, String),
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let specs: Vec<Fqn> =
        view.tests
            .model
            .projects()
            .get(&path)
            .map_or_else(Vec::new, |project: &ProjectTests| {
                project
                    .specs
                    .keys()
                    .filter(|name: &&Fqn| {
                        let parts: FqnParts<'_> = name.parts();
                        parts.namespace == owner.0 && parts.class == owner.1
                    })
                    .cloned()
                    .collect()
            });
    if !specs.is_empty() {
        run_one(
            view,
            path,
            Selection {
                specs,
                category: None,
            },
            window,
            cx,
        );
    }
}
pub(crate) fn run_category(
    view: &mut ExplorerView,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let category: Category = match Category::new(&view.tests.category) {
        Some(category) => category,
        None => return,
    };
    let requests: Vec<(PathBuf, Selection)> = view
        .tests
        .model
        .projects()
        .keys()
        .filter(|path: &&PathBuf| !super::super::uses_apex(path))
        .map(|path: &PathBuf| {
            (
                path.clone(),
                Selection {
                    specs: Vec::new(),
                    category: Some(category.clone()),
                },
            )
        })
        .collect();
    run_batch(view, requests, window, cx);
}

pub(crate) fn run_failed(
    view: &mut ExplorerView,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let failed: Vec<(PathBuf, Selection)> = view
        .tests
        .model
        .projects()
        .iter()
        .filter_map(|(path, project): (&PathBuf, &ProjectTests)| {
            let names: Vec<Fqn> = project
                .specs
                .iter()
                .filter_map(|(name, result): (&Fqn, &Option<bb_testkit::SpecResult>)| {
                    result
                        .as_ref()
                        .filter(|r: &&bb_testkit::SpecResult| {
                            r.outcome == bb_testkit::Outcome::Failed
                        })
                        .map(|_: &bb_testkit::SpecResult| name.clone())
                })
                .collect();
            (!names.is_empty()).then(|| {
                (
                    path.clone(),
                    Selection {
                        specs: names,
                        category: None,
                    },
                )
            })
        })
        .collect();
    run_batch(view, failed, window, cx);
}

fn run_one(
    view: &mut ExplorerView,
    path: PathBuf,
    selection: Selection,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    run_batch(view, vec![(path, selection)], window, cx);
}

fn run_batch(
    view: &mut ExplorerView,
    requests: Vec<(PathBuf, Selection)>,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    if view.tests.busy.is_some() || requests.is_empty() {
        return;
    }
    let stop: StopFlag = view.tests.begin_run(&requests);
    let root: PathBuf = view.root.clone();
    let settings: Option<PathBuf> = view.tests.chosen_settings();
    #[cfg(test)]
    let job: gpui::Task<Reports> = cx
        .background_executor()
        .spawn(async move { run_reports(&root, settings.as_deref(), requests, &stop) });
    #[cfg(not(test))]
    let receive: tokio::sync::oneshot::Receiver<Reports> =
        run_in_thread(root, settings, requests, stop);
    cx.spawn_in(
        window,
        async move |view: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext| {
            #[cfg(test)]
            let reports: Reports = job.await;
            #[cfg(not(test))]
            let reports: Reports =
                receive
                    .await
                    .unwrap_or_else(|error: tokio::sync::oneshot::error::RecvError| {
                        vec![(PathBuf::new(), Err(error.to_string()))]
                    });
            view.update_in(
                cx,
                |view: &mut ExplorerView, _: &mut Window, cx: &mut Context<ExplorerView>| {
                    apply_reports(view, reports, cx);
                },
            )
            .ok();
        },
    )
    .detach();
    cx.notify();
}

fn apply_reports(
    view: &mut ExplorerView,
    reports: Vec<(PathBuf, Result<RunReport, String>)>,
    cx: &mut Context<ExplorerView>,
) {
    view.tests.busy = None;
    view.tests.stop = None;
    view.tests.running_specs.clear();
    for (path, result) in reports {
        match result {
            Ok(report) => record_report(view, &path, report),
            Err(error) => view.tests.message = Some(error),
        }
    }
    cx.notify();
}

fn record_report(view: &mut ExplorerView, path: &Path, report: RunReport) {
    let counts: (usize, usize, usize, usize) = report.counts();
    let written: std::fmt::Result = write!(
        view.tests.output,
        "\n{} pass={} fail={} skip={} missing={}",
        path.display(),
        counts.0,
        counts.1,
        counts.2,
        counts.3
    );
    if written.is_err() {
        view.tests.message = Some(String::from("Output format failed"));
    }
    view.tests.output.push_str(&report.stdout);
    view.tests.output.push_str(&report.stderr);
    for leaf in report.leaves {
        match leaf {
            Leaf::Ran(result) => view.tests.model.record(path, result),
            Leaf::Missing(name) => view.tests.message = Some(format!("Missing result: {name}")),
        }
    }
}

pub(crate) fn stop(view: &mut ExplorerView, cx: &mut Context<ExplorerView>) {
    if let Some(flag) = &view.tests.stop {
        flag.stop();
        view.tests.message = Some(String::from("Stopping…"));
    }
    cx.notify();
}
