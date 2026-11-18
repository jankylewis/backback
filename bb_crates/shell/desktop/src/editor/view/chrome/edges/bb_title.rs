//! The title bar, drawn by backback as Rider's (BB-FEAT-090): after the
//! window buttons, the folder's name, then ← Back and → Forward (Cmd+[ /
//! Cmd+]), each grey when there is no place to go. A press then a move on
//! the bar moves the window; a double click zooms it once (BB-BUG-070).

use bb_ui::bb_theme::Theme;
use gpui::{
    div, prelude::*, px, ClickEvent, Context, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Window,
};

use crate::{color, ExplorerView};

/// The bar's height; the window buttons sit inside it.
const TITLE_BAR_PX: f32 = 32.0;
/// Room left for the window buttons.
const BUTTONS_PX: f32 = 78.0;

/// The bar's press state. The system drag starts only on a move while
/// pressed: started on the press, it took the second click of a double click
/// too and macOS zoomed again, so the window flipped back (BB-BUG-070).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TitleDrag {
    armed: bool,
}

impl TitleDrag {
    /// A press: a first click arms the drag; a double click never does.
    pub(crate) fn press(&mut self, clicks: usize) {
        self.armed = clicks < 2;
    }

    /// A move: true once, when the armed press turns into a window drag.
    /// A move with the button up disarms: the release fell outside the bar.
    pub(crate) fn moved(&mut self, pressed: bool) -> bool {
        std::mem::take(&mut self.armed) && pressed
    }

    /// A release: zoom when it ends a double click.
    pub(crate) fn release(&mut self, clicks: usize) -> bool {
        self.armed = false;
        clicks == 2
    }
}

/// The bar for `view`'s window.
pub(crate) fn title_bar(
    view: &ExplorerView,
    cx: &mut Context<ExplorerView>,
) -> gpui::Stateful<gpui::Div> {
    let theme: Theme = view.theme;
    let back: bool = view.tabs.history.can_back();
    let forward: bool = view.tabs.history.can_forward();
    let bar: gpui::Stateful<gpui::Div> = div()
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
        .border_color(color(theme.overlay));
    drag_and_zoom(bar, cx)
        .child(
            div()
                .pr_2()
                .text_color(color(theme.text_dim))
                .child(crate::launch::window_title(&view.root)),
        )
        .child(arrow(("nav-back", "←"), back, theme, cx))
        .child(arrow(("nav-forward", "→"), forward, theme, cx))
}

/// A press then a move drags the window; a double click zooms it once.
fn drag_and_zoom(
    bar: gpui::Stateful<gpui::Div>,
    cx: &mut Context<ExplorerView>,
) -> gpui::Stateful<gpui::Div> {
    bar.on_mouse_down(
        gpui::MouseButton::Left,
        cx.listener(
            |view: &mut ExplorerView,
             event: &MouseDownEvent,
             _: &mut Window,
             _: &mut Context<ExplorerView>| {
                view.title_drag.press(event.click_count);
            },
        ),
    )
    .on_mouse_move(cx.listener(
        |view: &mut ExplorerView,
         event: &MouseMoveEvent,
         window: &mut Window,
         _: &mut Context<ExplorerView>| {
            let pressed: bool = event.pressed_button == Some(gpui::MouseButton::Left);
            if view.title_drag.moved(pressed) {
                window.start_window_move();
            }
        },
    ))
    .on_mouse_up(
        gpui::MouseButton::Left,
        cx.listener(
            |view: &mut ExplorerView,
             event: &MouseUpEvent,
             window: &mut Window,
             _: &mut Context<ExplorerView>| {
                if view.title_drag.release(event.click_count) {
                    window.titlebar_double_click();
                }
            },
        ),
    )
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
        // A press or a double click on an arrow is not a drag or a zoom.
        .on_mouse_down(
            gpui::MouseButton::Left,
            |_: &MouseDownEvent, _: &mut Window, cx: &mut gpui::App| {
                cx.stop_propagation();
            },
        )
        .on_mouse_up(
            gpui::MouseButton::Left,
            |_: &MouseUpEvent, _: &mut Window, cx: &mut gpui::App| {
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
