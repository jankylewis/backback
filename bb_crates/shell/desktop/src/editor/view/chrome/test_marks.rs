//! Test methods in the editor: the gutter's ▶ (`target`) and, as Rider's
//! editor menu, a right-click inside a test method opens Run and Debug for
//! it (BB-FEAT-084).

use super::super::super::bb_tabs::Tab;
use crate::ExplorerView;
use bb_testkit::Fqn;
use gpui::InteractiveElement;
use std::path::PathBuf;

pub(crate) fn target(view: &ExplorerView, tab: &Tab, line: usize) -> Option<(PathBuf, Fqn)> {
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
                // BB-BUG-065: `method()` is the full name; the source has
                // only the method's own name.
                .find(|fqn: &&Fqn| short_name(fqn) == name)
                .map(|fqn: &Fqn| (path.clone(), fqn.clone()))
        },
    )
}

/// A test's method name without its namespace and class: `Fails` of
/// `Sample.Auth.OutcomeSpec.Fails`.
fn short_name(fqn: &Fqn) -> &str {
    fqn.method().rsplit('.').next().unwrap_or_default()
}

/// The test method holding `line`: the nearest one declared on or above it.
fn holding(tab: &Tab, line: usize) -> Option<usize> {
    tab.test_methods
        .iter()
        .map(|method: &bb_syntax::TestMethod| method.line)
        .filter(|declared: &usize| *declared <= line)
        .max()
}

/// `row` (editor line `line`) opens its test's Run / Debug menu on a
/// right-click; before the tests are listed, the TESTS panel opens instead.
pub(crate) fn on_right_click(
    row: gpui::Stateful<gpui::Div>,
    line: usize,
    cx: &mut gpui::Context<ExplorerView>,
) -> gpui::Stateful<gpui::Div> {
    row.on_mouse_down(
        gpui::MouseButton::Right,
        cx.listener(
            move |view: &mut ExplorerView,
                  event: &gpui::MouseDownEvent,
                  window: &mut gpui::Window,
                  cx: &mut gpui::Context<ExplorerView>| {
                let Some(tab): Option<&Tab> = view.tabs.active_tab() else {
                    return;
                };
                let Some(declared): Option<usize> = holding(tab, line) else {
                    return;
                };
                cx.stop_propagation();
                match target(view, tab, declared) {
                    Some((path, fqn)) => {
                        view.tests.menu = Some(crate::tests_panel::TestMenu {
                            path,
                            fqn,
                            at: event.position,
                        });
                        cx.notify();
                    }
                    None => crate::tests_panel::open(view, window, cx),
                }
            },
        ),
    )
}
