//! A test's right-click menu, as Rider's: Run and Debug. A press outside
//! closes it; picking an item closes it and does it.

use std::path::PathBuf;

use bb_testkit::Fqn;
use bb_ui::bb_theme::Theme;
use gpui::{anchored, deferred, div, prelude::*, AnyElement, Context, Window};

use crate::{color, ExplorerView};

/// The open menu: which test, and where the right-click was.
#[derive(Clone, Debug)]
pub(crate) struct TestMenu {
    pub(crate) path: PathBuf,
    pub(crate) fqn: Fqn,
    pub(crate) at: gpui::Point<gpui::Pixels>,
}

/// What a menu item does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pick {
    Run,
    Debug,
}

/// `row` opens the menu of the test at `path`, `fqn` on a right-click.
pub(super) fn on_right_click(
    row: gpui::Stateful<gpui::Div>,
    (path, fqn): (PathBuf, Fqn),
    cx: &mut Context<ExplorerView>,
) -> gpui::Stateful<gpui::Div> {
    row.on_mouse_down(
        gpui::MouseButton::Right,
        cx.listener(
            move |view: &mut ExplorerView,
                  event: &gpui::MouseDownEvent,
                  _: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                view.tests.menu = Some(TestMenu {
                    path: path.clone(),
                    fqn: fqn.clone(),
                    at: event.position,
                });
                cx.stop_propagation();
                cx.notify();
            },
        ),
    )
}

/// The open menu, drawn over everything.
pub(crate) fn menu(
    view: &ExplorerView,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> Option<AnyElement> {
    let open: &TestMenu = view.tests.menu.as_ref()?;
    let body: gpui::Stateful<gpui::Div> = div()
        .id("test-menu")
        .debug_selector(|| "test-menu".to_owned())
        .py_1()
        .min_w(gpui::px(140.0))
        .bg(color(theme.overlay))
        .text_color(color(theme.text))
        .on_mouse_down_out(cx.listener(
            |view: &mut ExplorerView,
             _: &gpui::MouseDownEvent,
             _: &mut Window,
             cx: &mut Context<ExplorerView>| {
                view.tests.menu = None;
                cx.notify();
            },
        ))
        .child(item("run", "▶ Run", Pick::Run, theme, cx))
        .child(item("debug", "Debug", Pick::Debug, theme, cx));
    Some(
        deferred(anchored().position(open.at).child(body))
            .with_priority(2)
            .into_any_element(),
    )
}

fn item(
    key: &'static str,
    label: &'static str,
    pick: Pick,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    div()
        .id(key)
        .debug_selector(move || format!("test-menu-{key}"))
        .px_3()
        .py_1()
        .hover(move |style: gpui::StyleRefinement| style.bg(color(theme.surface)))
        .child(label)
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                if let Some(open) = view.tests.menu.take() {
                    match pick {
                        Pick::Run => {
                            crate::tests_panel::run_spec(view, open.path, open.fqn, window, cx);
                        }
                        Pick::Debug => crate::run::bb_debug::session::control::debug_test(
                            view, open.path, open.fqn, window, cx,
                        ),
                    }
                }
                cx.notify();
            },
        ))
}
