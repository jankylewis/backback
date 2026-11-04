//! The Debug panel's title row: what the session does, then the two
//! exception ticks of VS Code's Breakpoints view, `[x] All Exceptions` and
//! `[ ] User-Unhandled Exceptions`; a click flips one.

use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, AnyElement, ClickEvent, Context, Window};

use crate::intel::view::one_line;
use crate::run::bb_debug::ExceptionKind;
use crate::{color, ExplorerView};

/// The title, then the ticks at the right edge.
pub(super) fn title_row(
    view: &ExplorerView,
    title: String,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    one_line(22.0)
        .px_2()
        .gap_3()
        .text_color(color(theme.text_dim))
        .child(title)
        .child(div().flex_1())
        .child(tick(
            ExceptionKind::All,
            view.debug.exceptions.all,
            theme,
            cx,
        ))
        .child(tick(
            ExceptionKind::UserUnhandled,
            view.debug.exceptions.user_unhandled,
            theme,
            cx,
        ))
        .child(crate::intel::view::close_x(
            "debug-panel-close",
            theme,
            cx,
            hide,
        ))
        .into_any_element()
}

/// The ✕: hides the panel; the session goes on (BB-BUG-047).
pub(crate) fn hide(view: &mut ExplorerView, _: &mut Window, _: &mut Context<ExplorerView>) {
    if let Some(live) = view.debug.live.as_mut() {
        live.hidden = true;
    }
}

/// One tick; a click flips it.
fn tick(kind: ExceptionKind, on: bool, theme: Theme, cx: &mut Context<ExplorerView>) -> AnyElement {
    let mark: &str = if on { "[x]" } else { "[ ]" };
    one_line(22.0)
        .id(kind.id())
        .debug_selector(move || kind.id().to_owned())
        .gap_1()
        .cursor_pointer()
        .hover(|el: gpui::StyleRefinement| el.bg(color(theme.overlay)))
        .text_color(color(if on { theme.text } else { theme.text_dim }))
        .child(mark.to_owned())
        .child(kind.label().to_owned())
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &ClickEvent,
                  _: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                crate::run::bb_debug::toggle_exceptions(view, kind);
                cx.notify();
            },
        ))
        .into_any_element()
}
