use super::super::bb_tabs::Tab;
use crate::ExplorerView;
use bb_testkit::Fqn;
use std::path::PathBuf;

pub(super) fn target(view: &ExplorerView, tab: &Tab, line: usize) -> Option<(PathBuf, Fqn)> {
    let name: &str = tab
        .test_methods
        .iter()
        .find(|method: &&bb_syntax::TestMethod| method.line == line)?
        .name
        .as_str();
    view.tests.model.projects().iter().find_map(
        |(path, project): (&PathBuf, &crate::tests_panel::model::ProjectTests)| {
            let dir: &std::path::Path = path.parent()?;
            if !tab.path.starts_with(dir) {
                return None;
            }
            project
                .specs
                .keys()
                .find(|fqn: &&Fqn| fqn.method() == name)
                .map(|fqn: &Fqn| (path.clone(), fqn.clone()))
        },
    )
}
