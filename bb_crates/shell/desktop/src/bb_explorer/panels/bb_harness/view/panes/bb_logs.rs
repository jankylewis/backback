use super::super::model::Server;
use super::button;
use crate::ExplorerView;
use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, uniform_list, AnyElement, Context, IntoElement, Window};
use std::ops::Range;

/// A longer log line shows its first characters only; Copy is not affected.
const LOG_ROW_CHARS: usize = 1000;

pub(super) fn logs(
    view: &ExplorerView,
    server: &Server,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    let count: usize = matching(&view.harness.search, server).len();
    div()
        .flex_1()
        .flex()
        .flex_col()
        .child(log_controls(view, theme, cx))
        .child(search_box(view, cx))
        .child(
            uniform_list(
                "logs-scroll",
                count,
                cx.processor(
                    |view: &mut ExplorerView,
                     range: Range<usize>,
                     _: &mut Window,
                     _: &mut Context<ExplorerView>| log_rows(view, range),
                ),
            )
            .track_scroll(&view.harness.log_scroll)
            .flex_1(),
        )
        .into_any_element()
}

/// Indices of the log lines that match the search, in order.
fn matching(search: &str, server: &Server) -> Vec<usize> {
    let needle: String = search.to_lowercase();
    server
        .logs
        .iter()
        .enumerate()
        .filter(|(_, line): &(usize, &String)| {
            needle.is_empty() || line.to_lowercase().contains(&needle)
        })
        .map(|(index, _): (usize, &String)| index)
        .collect()
}

fn log_rows(view: &ExplorerView, range: Range<usize>) -> Vec<AnyElement> {
    let Some(server): Option<&Server> = view
        .harness
        .selected
        .and_then(|index: usize| view.harness.servers.get(index))
    else {
        return Vec::new();
    };
    let lines: Vec<usize> = matching(&view.harness.search, server);
    range
        .filter_map(|row: usize| {
            lines
                .get(row)
                .and_then(|line: &usize| server.logs.get(*line))
        })
        .map(|line: &String| {
            let shown: String = line.chars().take(LOG_ROW_CHARS).collect();
            div()
                .px_2()
                .whitespace_nowrap()
                .child(shown)
                .into_any_element()
        })
        .collect()
}

fn log_controls(
    view: &ExplorerView,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    div()
        .flex()
        .gap_2()
        .child(button(
            "follow-logs".into(),
            if view.harness.follow {
                "Pause"
            } else {
                "Follow"
            },
            theme,
            cx,
            |view: &mut ExplorerView, cx: &mut Context<ExplorerView>| {
                view.harness.follow = !view.harness.follow;
                cx.notify();
            },
        ))
        .child(button(
            "clear-logs".into(),
            "Clear",
            theme,
            cx,
            |view: &mut ExplorerView, cx: &mut Context<ExplorerView>| {
                if let Some(index) = view.harness.selected {
                    if let Some(server) = view.harness.servers.get_mut(index) {
                        server.logs.clear();
                    }
                }
                cx.notify();
            },
        ))
}

fn search_box(view: &ExplorerView, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    div()
        .id("log-search")
        .px_2()
        .py_1()
        .child(format!("Search: {}", view.harness.search))
        .on_click(cx.listener(
            |view: &mut ExplorerView,
             _: &gpui::ClickEvent,
             window: &mut Window,
             cx: &mut Context<ExplorerView>| {
                view.harness.search_edit = true;
                window.focus(&view.harness_focus, cx);
                cx.notify();
            },
        ))
}
