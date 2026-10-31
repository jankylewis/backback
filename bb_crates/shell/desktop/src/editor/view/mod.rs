use bb_text::Document;
use bb_ui::bb_theme::DARK;
use gpui::{div, prelude::*, px, uniform_list, Context, FocusHandle, IntoElement, Window};

use super::bb_tabs::Tab;
mod bb_line;
use super::Tabs;
use crate::{color, ExplorerView};
use bb_line::{line_row, styled_line};

pub(crate) fn tab_bar(tabs: &Tabs, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    div()
        .h(px(36.0))
        .flex()
        .bg(color(DARK.surface))
        .children(
            tabs.items()
                .iter()
                .enumerate()
                .map(|(index, tab): (usize, &Tab)| {
                    tab_item(index, tab, tabs.active_index() == Some(index), cx)
                }),
        )
}

fn tab_item(
    index: usize,
    tab: &Tab,
    active: bool,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    let name: String = tab.path.file_name().map_or_else(
        || String::from("Untitled"),
        |part: &std::ffi::OsStr| part.to_string_lossy().into_owned(),
    );
    let dirty: &'static str = if tab.document.is_dirty() { " ●" } else { "" };
    div()
        .id(index)
        .h_full()
        .px_2()
        .flex()
        .items_center()
        .bg(color(if active { DARK.bg } else { DARK.surface }))
        .text_color(color(DARK.text))
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                view.tabs.switch(index);
                window.focus(&view.editor_focus, cx);
                cx.notify();
            },
        ))
        .child(div().text_color(color(DARK.icon_csharp)).child("C# "))
        .child(format!("{name}{dirty}"))
        .child(
            div()
                .id(format!("close-{index}"))
                .ml_2()
                .child("×")
                .on_click(cx.listener(
                    move |view: &mut ExplorerView,
                          _: &gpui::ClickEvent,
                          _: &mut Window,
                          cx: &mut Context<ExplorerView>| {
                        view.tabs.close(index);
                        cx.notify();
                    },
                )),
        )
}

pub(crate) fn editor_body(
    tabs: &Tabs,
    focus: &FocusHandle,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    let lines: usize = tabs.active().map_or(0, Document::len_lines);
    div()
        .id("editor-area")
        .flex_1()
        .bg(color(DARK.bg))
        .key_context("Editor")
        .track_focus(focus)
        .on_key_down(cx.listener(ExplorerView::editor_key_down))
        .child(
            uniform_list(
                "editor-lines",
                lines,
                cx.processor(
                    |view: &mut ExplorerView,
                     range: std::ops::Range<usize>,
                     _: &mut Window,
                     row_cx: &mut Context<ExplorerView>| {
                        range
                            .map(|index: usize| {
                                let tab: &Tab = match view.tabs.active_tab() {
                                    Some(tab) => tab,
                                    None => return line_row(index, Vec::new(), false, row_cx),
                                };
                                let current: bool = tab.document.cursor_line() == index;
                                line_row(index, styled_line(tab, index), current, row_cx)
                            })
                            .collect::<Vec<_>>()
                    },
                ),
            )
            .h_full(),
        )
        .when(
            tabs.pending_close(),
            |element: gpui::Stateful<gpui::Div>| element.child(super::prompt::close_prompt(cx)),
        )
}
