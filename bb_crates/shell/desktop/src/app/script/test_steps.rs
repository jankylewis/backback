//! Tests panel steps: the handlers a row click, its ▶ and its ■ run.

use std::path::PathBuf;

use bb_testkit::Fqn;
use gpui::{AsyncWindowContext, Context, WeakEntity, Window};

use crate::ExplorerView;

/// The first listed spec whose name holds `name`, with its project.
fn find(view: &ExplorerView, name: &str) -> Option<(PathBuf, Fqn)> {
    view.tests.model.projects().iter().find_map(
        |(path, project): (&PathBuf, &crate::tests_panel::model::ProjectTests)| {
            project
                .specs
                .keys()
                .find(|fqn: &&Fqn| fqn.as_str().contains(name))
                .map(|fqn: &Fqn| (path.clone(), fqn.clone()))
        },
    )
}

pub(super) fn open(view: &WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext, name: &str) {
    let _gone: gpui::Result<()> = view.update_in(
        cx,
        |v: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>| match find(
            v, name,
        ) {
            Some((path, fqn)) => crate::tests_panel::open_spec(v, &path, &fqn, window, cx),
            None => eprintln!("bb_script no spec named {name}"),
        },
    );
}

pub(super) fn run(view: &WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext, name: &str) {
    let _gone: gpui::Result<()> = view.update_in(
        cx,
        |v: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>| match find(
            v, name,
        ) {
            Some((path, fqn)) => crate::tests_panel::run_spec(v, path, fqn, window, cx),
            None => eprintln!("bb_script no spec named {name}"),
        },
    );
}

pub(super) fn stop(view: &WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext) {
    let _gone: gpui::Result<()> = view.update_in(
        cx,
        |v: &mut ExplorerView, _: &mut Window, cx: &mut Context<ExplorerView>| {
            crate::tests_panel::stop(v, cx);
        },
    );
}
