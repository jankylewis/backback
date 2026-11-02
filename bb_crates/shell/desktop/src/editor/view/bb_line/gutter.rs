use super::{severity_ink, Ink};
use crate::{color, ExplorerView};
use bb_testkit::Fqn;
use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, Context, Window};
use std::path::PathBuf;

pub(crate) struct TestGutter {
    pub(crate) target: Option<(PathBuf, Fqn)>,
}

/// The line number (on a red tag when the line has a breakpoint; a click
/// toggles it) and, when the line has a diagnostic, a dot in its ink.
pub(super) fn gutter_cell(
    index: usize,
    (gutter, breakpoint): (Option<Ink>, bool),
    test: Option<TestGutter>,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> gpui::Div {
    div()
        .w(px(crate::settings::typography(cx).gutter_width()))
        .flex_none()
        .whitespace_nowrap()
        .flex()
        .text_color(color(theme.text_dim))
        .child(number(index, breakpoint, theme, cx))
        .child(
            div()
                .w(px(10.0))
                .when_some(gutter, |dot: gpui::Div, s: Ink| {
                    dot.text_color(color(severity_ink(s, theme))).child("●")
                }),
        )
        .when_some(test, |cell: gpui::Div, test: TestGutter| {
            cell.child(
                div()
                    .id(format!("test-gutter-{index}"))
                    .cursor_pointer()
                    .text_color(color(theme.accent))
                    .child("▶")
                    .on_click(cx.listener(
                        move |view: &mut ExplorerView,
                              _: &gpui::ClickEvent,
                              window: &mut Window,
                              cx: &mut Context<ExplorerView>| {
                            cx.stop_propagation();
                            if let Some((project, fqn)) = &test.target {
                                crate::tests_panel::run_spec(
                                    view,
                                    project.clone(),
                                    fqn.clone(),
                                    window,
                                    cx,
                                );
                            } else {
                                crate::tests_panel::open(view, window, cx);
                            }
                        },
                    )),
            )
        })
}

/// The line number: a click toggles the line's breakpoint without moving
/// the cursor (the row's own mouse-down never sees it).
fn number(
    index: usize,
    breakpoint: bool,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(("line-number", index))
        .debug_selector(move || format!("line-number-{index}"))
        .cursor_pointer()
        .rounded(px(3.0))
        .when(breakpoint, |tag: gpui::Stateful<gpui::Div>| {
            tag.bg(color(theme.error))
                .text_color(color(theme.bg))
                .debug_selector(move || format!("breakpoint-{index}"))
        })
        .child(format!("{:>4}", index.saturating_add(1)))
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(
                move |view: &mut ExplorerView,
                      _: &gpui::MouseDownEvent,
                      _: &mut Window,
                      cx: &mut Context<ExplorerView>| {
                    cx.stop_propagation();
                    let path: Option<PathBuf> = view
                        .tabs
                        .active_tab()
                        .map(|tab: &crate::editor::Tab| tab.path.clone());
                    if let Some(path) = path {
                        crate::run::bb_debug::toggle_line(view, &path, index);
                    }
                    cx.notify();
                },
            ),
        )
}
