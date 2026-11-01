use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, AnyElement, Context, IntoElement, Window};

use crate::{color, ExplorerView};

pub(super) fn label(text: impl Into<String>, theme: Theme) -> AnyElement {
    div()
        .min_h(px(24.0))
        .px_2()
        .text_color(color(theme.text_dim))
        .child(text.into())
        .into_any_element()
}

pub(super) fn detail(
    view: &ExplorerView,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    let text: String = if view.tests.output_open {
        view.tests.output.clone()
    } else {
        view.tests.detail.as_ref().map_or_else(
            || view.tests.message.clone().unwrap_or_default(),
            |result: &bb_testkit::SpecResult| {
                format!(
                    "{} ms  {}\n{}",
                    result.duration.as_millis(),
                    result.message.as_deref().unwrap_or_default(),
                    result.stack.as_deref().unwrap_or_default()
                )
            },
        )
    };
    div()
        .id("test-detail")
        .debug_selector(|| String::from("test-detail"))
        .h(px(120.0))
        .overflow_y_scroll()
        .border_t_1()
        .border_color(color(theme.border))
        .child(text)
        .on_click(cx.listener(
            |view: &mut ExplorerView,
             _: &gpui::ClickEvent,
             window: &mut Window,
             cx: &mut Context<ExplorerView>| {
                super::super::navigation::jump(view, window, cx);
            },
        ))
}
