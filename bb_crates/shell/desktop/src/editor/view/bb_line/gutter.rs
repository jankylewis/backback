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
    (gutter, breakpoint, stopped): (Option<Ink>, bool, bool),
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
        .child(mark(index, gutter, stopped, theme))
        .when_some(test, |cell: gpui::Div, test: TestGutter| {
            cell.child(
                div()
                    .id(format!("test-gutter-{index}"))
                    .debug_selector(move || format!("test-gutter-{index}"))
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

/// The slot after the number: the paused line's arrow, else a diagnostic
/// dot in its ink.
fn mark(index: usize, gutter: Option<Ink>, stopped: bool, theme: Theme) -> gpui::AnyElement {
    let slot: gpui::Div = div().w(px(10.0));
    if stopped {
        // BB-FEAT-086: drag the arrow to a line to run that line next, as Rider.
        return slot
            .id(("stopped-arrow", index))
            .text_color(color(theme.warn))
            .debug_selector(move || format!("stopped-line-{index}"))
            .cursor_grab()
            .child("➜")
            .on_drag(
                DraggedArrow(theme),
                |arrow: &DraggedArrow,
                 _: gpui::Point<gpui::Pixels>,
                 _: &mut Window,
                 cx: &mut gpui::App| {
                    cx.new(|_: &mut Context<DraggedArrow>| *arrow)
                },
            )
            .into_any_element();
    }
    slot.when_some(gutter, |dot: gpui::Div, s: Ink| {
        dot.text_color(color(severity_ink(s, theme))).child("●")
    })
    .into_any_element()
}

/// The execution arrow while it is dragged; it follows the pointer.
#[derive(Clone, Copy)]
pub(crate) struct DraggedArrow(Theme);

impl gpui::Render for DraggedArrow {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().text_color(color(self.0.warn)).child("➜")
    }
}

/// Line `line` takes the dragged arrow: the caret goes there and Set Next
/// Statement runs that line next (Ctrl+Shift+F10, BBEOD05).
pub(super) fn arrow_drop(
    row: gpui::Stateful<gpui::Div>,
    line: usize,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> gpui::Stateful<gpui::Div> {
    row.drag_over::<DraggedArrow>(
        move |style: gpui::StyleRefinement, _: &DraggedArrow, _: &mut Window, _: &mut gpui::App| {
            style.bg(gpui::Rgba {
                a: 0.28,
                ..color(theme.warn)
            })
        },
    )
    .on_drop(cx.listener(
        move |view: &mut ExplorerView,
              _: &DraggedArrow,
              _: &mut Window,
              cx: &mut Context<ExplorerView>| {
            use crate::run::bb_debug::session::control;
            if !control::can_set_next(view) {
                view.report(&"Set Next Statement: this debugger cannot move the next statement");
                return;
            }
            if let Some(doc) = view.tabs.active_mut() {
                if let Some(start) = doc.line_start(line) {
                    let _moved: Result<(), _> = doc.set_cursor(start);
                }
            }
            control::set_next(view);
            cx.notify();
        },
    ))
}
