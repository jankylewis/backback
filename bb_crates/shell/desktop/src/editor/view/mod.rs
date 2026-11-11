use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, uniform_list, Context, FocusHandle, IntoElement, Window};

mod bb_line;
mod pane;
pub(crate) mod wrap;
#[cfg(test)]
pub(crate) use pane::lines_tests as pane_tests;
pub(crate) mod chrome;
use super::bb_tabs::Tab;
use super::Tabs;
use crate::intel::view::marks::{line_marks, Mark};
use crate::{color, ExplorerView};
pub(crate) use bb_line::Columns;
use bb_line::{line_row, styled_part, LineRow, TestGutter};
pub(crate) use chrome::bb_scrollbar;
use chrome::{status, test_marks};
pub(crate) use pane::drag::{bottom_handle, handle, Edge, Lines, HANDLE_PX};
pub(crate) use pane::{area as editor_area, layout as layout_panes, panes};
pub(crate) use status::{build, status_bar};

/// BB-BUG-060: a press in the editor that takes focus back from the tree
/// or a panel shows the file in the Explorer again, as VS Code does.
fn back_in_the_file(
    view: &mut ExplorerView,
    _: &gpui::MouseDownEvent,
    window: &mut gpui::Window,
    cx: &mut Context<ExplorerView>,
) {
    if !view.editor_focus.is_focused(window) {
        crate::navigation::reveal_again(view);
        window.focus(&view.editor_focus, cx);
        cx.notify();
    }
}

pub(super) fn editor_body(view: &ExplorerView, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    let (tabs, focus, theme): (&Tabs, &FocusHandle, Theme) =
        (&view.tabs, &view.editor_focus, view.theme);
    div()
        .id("editor-area")
        .debug_selector(|| "editor-area".to_owned())
        .flex_1()
        .relative()
        .bg(color(theme.bg))
        .key_context("Editor")
        .track_focus(focus)
        .on_key_down(cx.listener(ExplorerView::editor_key_down))
        .capture_any_mouse_down(cx.listener(back_in_the_file))
        .map(|area: gpui::Stateful<gpui::Div>| super::link::on_editor(area, view, cx))
        .on_action(cx.listener(super::go_back))
        .on_action(cx.listener(crate::run::build::on_build))
        .on_action(cx.listener(crate::quick_open::on_go_to_symbol))
        .on_action(cx.listener(super::go_forward))
        .map(|area: gpui::Stateful<gpui::Div>| crate::editor::on_tab_actions(area, cx))
        .on_scroll(cx.listener(pane::bb_hscroll::on_wheel))
        .child(focused_lines(view, cx))
        .children(bb_scrollbar::bars(view, view.tabs.focused(), cx))
        .children(crate::intel::view::bb_popups::hover(
            view.intel.hover.as_ref(),
            theme,
        ))
        .when(
            tabs.pending_close(),
            |element: gpui::Stateful<gpui::Div>| {
                element.child(super::prompt::close_prompt(theme, cx))
            },
        )
}

/// The focused pane's lines; each frame first follows the view and the
/// sideways scroll.
fn focused_lines(view: &ExplorerView, cx: &mut Context<ExplorerView>) -> gpui::UniformList {
    let lines: usize = view.tabs.active_tab().map_or(0, wrap::row_count);
    uniform_list(
        "editor-lines",
        lines,
        cx.processor(
            |view: &mut ExplorerView,
             range: std::ops::Range<usize>,
             window: &mut Window,
             row_cx: &mut Context<ExplorerView>| {
                follow_view(view, &range, window, row_cx);
                if view
                    .tabs
                    .active_tab()
                    .is_none_or(|tab: &Tab| tab.wrap.is_none())
                {
                    pane::bb_hscroll::before_rows(view, &range, window, row_cx);
                }
                range
                    .map(|index: usize| focused_line(view, index, window, row_cx))
                    .collect::<Vec<_>>()
            },
        ),
    )
    .track_scroll(view.tabs.scroll())
    .h_full()
}

/// A large file parses only lines near the view: move the window with it.
fn follow_view(
    view: &mut ExplorerView,
    visible: &std::ops::Range<usize>,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let moved: Option<std::path::PathBuf> = view.tabs.active_tab_mut().and_then(|tab: &mut Tab| {
        let lines: std::ops::Range<usize> = wrap::lines_of(tab, visible);
        tab.show_lines(&lines).then(|| tab.path.clone())
    });
    if let Some(path) = moved {
        super::input::schedule_parse(view, path, window, cx);
    }
}

fn focused_line(
    view: &ExplorerView,
    index: usize,
    window: &Window,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    editor_line(view, view.tabs.focused(), index, window, cx)
}

/// Line `index` of `side`'s active tab. Only the focused pane draws the
/// cursor, its line and the popups at it (BB-BUG-040).
pub(super) fn editor_line(
    view: &ExplorerView,
    side: super::Group,
    index: usize,
    window: &Window,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    let Some(tab): Option<&Tab> = view.tabs.active_tab_of(side) else {
        return line_row(LineRow::blank(index), &[], view.theme, cx);
    };
    let part: wrap::Part = wrap::part(tab, index);
    let line: usize = part.line;
    let current: bool = side == view.tabs.focused() && tab.document.cursor_line() == line;
    let marks: Vec<Mark> = line_marks(&tab.document, view.intel.diagnostics_of(&tab.path), line);
    let fonts: bb_ui::Typography = crate::settings::typography(cx);
    let caret: Option<f32> = (current && wrap::row_at(tab, line) == index)
        .then(|| part.indent + caret_x_in(tab, &part, window, &fonts));
    let scroll_x: f32 = if tab.wrap.is_some() {
        0.0
    } else {
        tab.scroll_x
    };
    let overlay: Option<gpui::AnyElement> = caret.and_then(|x: f32| {
        crate::intel::view::bb_popups::at_cursor(
            &view.intel,
            fonts.gutter_width() + x - scroll_x,
            view.theme,
        )
    });
    let row: LineRow = LineRow {
        index: line,
        at: (index, (!part.first).then_some(part.indent)),
        current,
        gutter: crate::intel::view::marks::worst(&marks),
        breakpoint: view.debug.breakpoints.has(&tab.path, line),
        stopped: crate::run::bb_debug::stopped_line(view, &tab.path) == Some(line),
        test: test_gutter(view, tab, line),
        overlay,
        caret,
        scroll_x,
        braces: chrome::braces::quads(tab, &part, (scroll_x, view.theme), window, &fonts),
    };
    let shown: Option<(bb_text::CharIdx, bb_text::CharIdx)> = super::link::shown(view, &tab.path);
    let runs: Vec<bb_line::Run> = styled_part(tab, line, part.columns.clone(), &marks, shown);
    line_row(row, &runs, view.theme, cx)
}

/// The ▶ of a test method declared on `line`.
fn test_gutter(view: &ExplorerView, tab: &Tab, line: usize) -> Option<TestGutter> {
    tab.test_methods
        .iter()
        .any(|method: &bb_syntax::TestMethod| method.line == line)
        .then(|| TestGutter {
            target: test_marks::target(view, tab, line),
        })
}

/// The cursor's x in `part` of its line, from the part's left edge.
fn caret_x_in(tab: &Tab, part: &wrap::Part, window: &Window, fonts: &bb_ui::Typography) -> f32 {
    let start: usize = tab
        .document
        .line_start(part.line)
        .map_or(0, bb_text::CharIdx::raw);
    let column: usize = tab.document.cursor().raw().saturating_sub(start);
    let text: String = wrap::part_text(tab, part);
    bb_line::Columns::shape(window, &text, fonts).x_of(column.saturating_sub(part.columns.start))
}

/// The cursor's x on its line, measured on the shaped line the row paints.
pub(super) fn caret_x(tab: &Tab, index: usize, window: &Window, fonts: &bb_ui::Typography) -> f32 {
    let start: usize = tab
        .document
        .line_start(index)
        .map_or(0, bb_text::CharIdx::raw);
    let column: usize = tab.document.cursor().raw().saturating_sub(start);
    let text: String = tab.document.line(index).unwrap_or_default();
    bb_line::Columns::shape(window, text.trim_end_matches(['\r', '\n']), fonts).x_of(column)
}

#[cfg(test)]
mod tests {
    fn sources(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for item in std::fs::read_dir(dir).unwrap() {
            let path: std::path::PathBuf = item.unwrap().path();
            if path.is_dir() {
                sources(&path, out);
            } else if path
                .extension()
                .is_some_and(|ext: &std::ffi::OsStr| ext == "rs")
            {
                out.push(path);
            }
        }
    }

    #[test]
    fn every_window_error_reaches_the_status_bar_not_only_stderr() {
        let root: std::path::PathBuf = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files: Vec<std::path::PathBuf> = Vec::new();
        sources(&root, &mut files);
        let stderr_only: Vec<String> = files
            .iter()
            .filter(|path: &&std::path::PathBuf| !path.ends_with("bb_launch/mod.rs"))
            .filter(|path: &&std::path::PathBuf| {
                std::fs::read_to_string(path)
                    .unwrap()
                    .contains(concat!("eprintln!(\"{", "error}\")"))
            })
            .map(|path: &std::path::PathBuf| path.display().to_string())
            .collect();
        assert!(files.len() > 10);
        assert_eq!(stderr_only, Vec::<String>::new());
    }
}
