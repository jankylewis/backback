use super::super::model::Server;
use super::button;
use crate::ExplorerView;
use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, AnyElement, Context, IntoElement, StatefulInteractiveElement, Window};

pub(super) fn logs(
    view: &ExplorerView,
    server: &Server,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    let search: String = view.harness.search.to_lowercase();
    let visible: Vec<AnyElement> = server
        .logs
        .iter()
        .filter(|line: &&String| line.to_lowercase().contains(&search))
        .map(|line: &String| div().px_2().child(line.clone()).into_any_element())
        .collect();
    div()
        .flex_1()
        .flex()
        .flex_col()
        .child(log_controls(view, theme, cx))
        .child(search_box(view, cx))
        .child(
            div()
                .id("logs-scroll")
                .flex_1()
                .overflow_y_scroll()
                .track_scroll(&view.harness.log_scroll)
                .children(visible),
        )
        .into_any_element()
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
