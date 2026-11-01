use super::{severity_ink, Ink};
use crate::{color, ExplorerView};
use bb_testkit::Fqn;
use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, Context, Window};
use std::path::PathBuf;

pub(crate) struct TestGutter {
    pub(crate) target: Option<(PathBuf, Fqn)>,
}

/// The line number and, when the line has a diagnostic, a dot in its ink.
pub(super) fn gutter_cell(
    index: usize,
    gutter: Option<Ink>,
    test: Option<TestGutter>,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> gpui::Div {
    div()
        .w(px(50.0))
        .flex()
        .text_color(color(theme.text_dim))
        .child(format!("{:>4}", index.saturating_add(1)))
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
