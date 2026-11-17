//! The title bar, drawn by backback as Rider's (BB-FEAT-090): after the
//! window buttons, the folder's name, then ← Back and → Forward (Cmd+[ /
//! Cmd+]), each grey when there is no place to go. A press on the bar moves
//! the window; a double click zooms it, as the system's bar does.

use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, ClickEvent, Context, MouseDownEvent, Window};

use crate::{color, ExplorerView};

/// The bar's height; the window buttons sit inside it.
const TITLE_BAR_PX: f32 = 32.0;
/// Room left for the window buttons.
const BUTTONS_PX: f32 = 78.0;

/// The bar for `view`'s window.
pub(crate) fn title_bar(
    view: &ExplorerView,
    cx: &mut Context<ExplorerView>,
) -> gpui::Stateful<gpui::Div> {
    let theme: Theme = view.theme;
    let back: bool = view.tabs.history.can_back();
    let forward: bool = view.tabs.history.can_forward();
    div()
        .id("title-bar")
        .debug_selector(|| "title-bar".to_owned())
        .h(px(TITLE_BAR_PX))
        .flex_none()
        .flex()
        .items_center()
        .gap_1()
        .pl(px(BUTTONS_PX))
        .bg(color(theme.surface))
        .border_b_1()
        .border_color(color(theme.overlay))
        .on_mouse_down(
            gpui::MouseButton::Left,
            |event: &MouseDownEvent, window: &mut Window, _: &mut gpui::App| {
                if event.click_count >= 2 {
                    window.titlebar_double_click();
                } else {
                    window.start_window_move();
                }
            },
        )
        .child(
            div()
                .pr_2()
                .text_color(color(theme.text_dim))
                .child(crate::launch::window_title(&view.root)),
        )
        .child(arrow(("nav-back", "←"), back, theme, cx))
        .child(arrow(("nav-forward", "→"), forward, theme, cx))
}

/// One arrow: live when there is a place to go, else grey and inert.
fn arrow(
    (id, glyph): (&'static str, &'static str),
    live: bool,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> gpui::Stateful<gpui::Div> {
    let button: gpui::Stateful<gpui::Div> = div()
        .id(id)
        .debug_selector(move || {
            if live {
                id.to_owned()
            } else {
                format!("{id}-off")
            }
        })
        .px_1()
        .rounded(px(4.0))
        // A press on an arrow is not a window drag.
        .on_mouse_down(
            gpui::MouseButton::Left,
            |_: &MouseDownEvent, _: &mut Window, cx: &mut gpui::App| {
                cx.stop_propagation();
            },
        )
        .child(glyph);
    if !live {
        return button.text_color(color(theme.text_dim)).opacity(0.4);
    }
    let back: bool = id == "nav-back";
    button
        .text_color(color(theme.text))
        .cursor_pointer()
        .hover(|style: gpui::StyleRefinement| style.bg(color(theme.overlay)))
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                crate::editor::go(view, back, window, cx);
            },
        ))
}
