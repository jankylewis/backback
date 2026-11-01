//! Tool result rows: only the visible rows are built and shaped.

use std::ops::Range;

use super::super::model::{RowText, Server};
use super::button;
use crate::{color, ExplorerView};
use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, uniform_list, AnyElement, Context, IntoElement, Window};

const RESULT_HEIGHT: f32 = 320.0;

pub(super) fn result_view(
    view: &ExplorerView,
    rows: usize,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    div()
        .child(
            uniform_list(
                "tool-result",
                rows,
                cx.processor(
                    |view: &mut ExplorerView,
                     range: Range<usize>,
                     _: &mut Window,
                     _: &mut Context<ExplorerView>| result_rows(view, range),
                ),
            )
            .track_scroll(&view.harness.result_scroll)
            .h(px(RESULT_HEIGHT))
            .px_2()
            .text_color(color(theme.text)),
        )
        .child(button("copy-result".into(), "Copy", theme, cx, copy_result))
}

fn selected_result(view: &ExplorerView) -> Option<&RowText> {
    view.harness
        .selected
        .and_then(|index: usize| view.harness.servers.get(index))
        .and_then(|server: &Server| server.result.as_ref())
}

fn result_rows(view: &ExplorerView, range: Range<usize>) -> Vec<AnyElement> {
    let text: Option<&RowText> = selected_result(view);
    range
        .map(|index: usize| {
            let row: String = text.map_or("", |rows: &RowText| rows.row(index)).to_owned();
            div().whitespace_nowrap().child(row).into_any_element()
        })
        .collect()
}

fn copy_result(view: &mut ExplorerView, cx: &mut Context<ExplorerView>) {
    if let Some(text) = selected_result(view) {
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(text.as_str().to_owned()));
    }
}
