use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, Context, IntoElement, Window};

use crate::{color, ExplorerView};

/// The foot of the editor: the latest open or save error on the left, the
/// C# server's load state on the right.
pub(crate) fn status_bar(
    status: Option<&str>,
    intel: &str,
    stopped: bool,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    div()
        .h(px(24.0))
        .px_2()
        .flex()
        .items_center()
        .justify_between()
        .bg(color(theme.overlay))
        .text_color(color(theme.text))
        .child(
            crate::intel::view::one_line(24.0)
                .flex_1()
                .min_w(px(0.0))
                .child(status.unwrap_or_default().to_owned()),
        )
        .child(
            div()
                .flex_none()
                .flex()
                .items_center()
                .pl_2()
                .text_color(color(theme.text_dim))
                .child(if stopped {
                    String::from("C# server stopped — ")
                } else {
                    intel.to_owned()
                })
                .when(stopped, |label: gpui::Div| {
                    label.child(restart_button(theme, cx))
                }),
        )
}

fn restart_button(theme: Theme, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    div()
        .id("restart-server")
        .cursor_pointer()
        .text_color(color(theme.icon_csharp))
        .child("Restart")
        .on_click(cx.listener(
            |view: &mut ExplorerView,
             _: &gpui::ClickEvent,
             window: &mut Window,
             cx: &mut Context<ExplorerView>| {
                crate::intel::glue::restart(view, window, cx);
            },
        ))
}
