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
        .children(
            text.lines()
                .enumerate()
                .map(|(index, line): (usize, &str)| stack_line(index, line, theme, cx)),
        )
        .on_click(cx.listener(
            |view: &mut ExplorerView,
             _: &gpui::ClickEvent,
             window: &mut Window,
             cx: &mut Context<ExplorerView>| {
                super::super::navigation::jump(view, window, cx);
            },
        ))
}

/// One line of the detail or output (BBEOD18): a stack frame with a source
/// is a link to its call site; anything else is plain text.
fn stack_line(
    index: usize,
    line: &str,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    let row: gpui::Stateful<gpui::Div> = div()
        .id(("stack-line", index))
        .whitespace_nowrap()
        .child(line.to_owned());
    let Some((path, at)): Option<(std::path::PathBuf, usize)> =
        super::super::navigation::frame_place(line)
    else {
        return row.into_any_element();
    };
    row.debug_selector(move || format!("stack-link-{index}"))
        .text_color(color(theme.accent))
        .cursor_pointer()
        .hover(|el: gpui::StyleRefinement| el.underline())
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                cx.stop_propagation();
                super::super::navigation::jump_to(view, (&path, at), window, cx);
                cx.notify();
            },
        ))
        .into_any_element()
}
