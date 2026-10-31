#[path = "view/logs.rs"]
mod bb_logs;
mod bb_tools;
use bb_logs::logs;
pub(crate) use bb_tools::key_down;
use bb_tools::tools;
use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, AnyElement, Context, IntoElement, Window};

use super::model::{Server, ServerStatus};
use super::Tab;
use crate::{color, ExplorerView};

pub(crate) fn panel(view: &ExplorerView, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    let theme: Theme = view.theme;
    let selected: Option<usize> = view.harness.selected;
    div()
        .id("harness-panel")
        .w(px(350.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(color(theme.surface))
        .text_color(color(theme.text))
        .key_context("Harness")
        .track_focus(&view.harness_focus)
        .on_key_down(cx.listener(ExplorerView::harness_key_down))
        .child(div().px_3().py_2().child("HARNESS"))
        .children(
            view.harness
                .servers
                .iter()
                .enumerate()
                .map(|(index, server): (usize, &Server)| {
                    server_row(index, server, selected == Some(index), theme, cx).into_any_element()
                })
                .collect::<Vec<AnyElement>>(),
        )
        .child(detail(view, theme, cx))
}

fn server_row(
    index: usize,
    server: &Server,
    selected: bool,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    let state: &'static str = match server.status {
        ServerStatus::Stopped => "○ stopped",
        ServerStatus::Starting => "◌ starting",
        ServerStatus::Stopping => "◌ stopping",
        ServerStatus::Running => "● running",
        ServerStatus::Crashed(_) => "● crashed",
    };
    let name: String = server.name.clone();
    div()
        .id(format!("server-{index}"))
        .px_2()
        .py_1()
        .flex()
        .flex_col()
        .bg(color(if selected {
            theme.overlay
        } else {
            theme.surface
        }))
        .child(
            div()
                .flex()
                .gap_2()
                .child(name)
                .child(div().text_color(color(theme.text_dim)).child(state)),
        )
        .child(server_controls(index, theme, cx))
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                view.harness.selected = Some(index);
                view.harness.tool = None;
                window.focus(&view.harness_focus, cx);
                cx.notify();
            },
        ))
}

fn server_controls(index: usize, theme: Theme, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    div()
        .flex()
        .gap_2()
        .child(button(
            format!("start-{index}"),
            "Start",
            theme,
            cx,
            move |view: &mut ExplorerView, cx: &mut Context<ExplorerView>| {
                view.harness.selected = Some(index);
                view.harness.start(index);
                cx.notify();
            },
        ))
        .child(button(
            format!("stop-{index}"),
            "Stop",
            theme,
            cx,
            move |view: &mut ExplorerView, cx: &mut Context<ExplorerView>| {
                view.harness.stop(index);
                cx.notify();
            },
        ))
        .child(button(
            format!("restart-{index}"),
            "Restart",
            theme,
            cx,
            move |view: &mut ExplorerView, cx: &mut Context<ExplorerView>| {
                view.harness.start(index);
                cx.notify();
            },
        ))
}

fn detail(view: &ExplorerView, theme: Theme, cx: &mut Context<ExplorerView>) -> AnyElement {
    let Some(index): Option<usize> = view.harness.selected else {
        return div()
            .px_3()
            .py_2()
            .child("Select an MCP server")
            .into_any_element();
    };
    let Some(server): Option<&Server> = view.harness.servers.get(index) else {
        return div().into_any_element();
    };
    let error: Option<String> = server.error.clone();
    div()
        .flex_1()
        .flex()
        .flex_col()
        .overflow_hidden()
        .child(tabs(theme, cx))
        .when_some(error, |element: gpui::Div, message: String| {
            element.child(
                div()
                    .px_2()
                    .text_color(color(theme.icon_csharp))
                    .child(message),
            )
        })
        .child(match view.harness.tab {
            Tab::Tools => tools(view, server, theme, cx),
            Tab::Log => logs(view, server, theme, cx),
        })
        .into_any_element()
}

fn tabs(theme: Theme, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    div()
        .flex()
        .gap_2()
        .child(button(
            "tools-tab".into(),
            "Tools",
            theme,
            cx,
            |view: &mut ExplorerView, cx: &mut Context<ExplorerView>| {
                view.harness.tab = Tab::Tools;
                cx.notify();
            },
        ))
        .child(button(
            "log-tab".into(),
            "Log",
            theme,
            cx,
            |view: &mut ExplorerView, cx: &mut Context<ExplorerView>| {
                view.harness.tab = Tab::Log;
                cx.notify();
            },
        ))
}

fn button(
    id: String,
    label: &'static str,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
    action: impl Fn(&mut ExplorerView, &mut Context<ExplorerView>) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .px_2()
        .py_1()
        .bg(color(theme.overlay))
        .child(label)
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  _: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                action(view, cx);
            },
        ))
}
