use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, Context, IntoElement, MouseButton, Window};

use crate::{color, ExplorerView};

pub(crate) fn close_prompt(theme: Theme, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    div()
        .absolute()
        .left(px(40.0))
        .top(px(60.0))
        .p_4()
        .bg(color(theme.overlay))
        .text_color(color(theme.text))
        .child("Save changes before closing?")
        .child(prompt_button("Save", cx, save_prompt))
        .child(prompt_button("Discard", cx, discard_prompt))
        .child(prompt_button("Cancel", cx, cancel_prompt))
}

fn save_prompt(view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>) {
    match view.tabs.save_close() {
        Ok(()) => view.finish_close(window, cx),
        Err(error) => view.report(&error),
    }
}

fn discard_prompt(view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>) {
    view.tabs.discard_close();
    view.finish_close(window, cx);
}

fn cancel_prompt(view: &mut ExplorerView, _: &mut Window, cx: &mut Context<ExplorerView>) {
    view.tabs.cancel_close();
    view.closing_window = false;
    view.quit_after_close = false;
    cx.notify();
}

fn prompt_button(
    text: &'static str,
    cx: &mut Context<ExplorerView>,
    action: fn(&mut ExplorerView, &mut Window, &mut Context<ExplorerView>),
) -> impl IntoElement {
    div().p_2().child(text).on_mouse_down(
        MouseButton::Left,
        cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::MouseDownEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                action(view, window, cx);
                cx.notify();
            },
        ),
    )
}

/// Quit the app; dirty tabs ask first, like closing the window.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct Quit;

impl ExplorerView {
    pub(crate) fn quit(&mut self, _: &Quit, _: &mut Window, cx: &mut Context<Self>) {
        self.quit_after_close = true;
        if self.handle_window_close(cx) {
            cx.quit();
        }
    }

    pub(crate) fn handle_window_close(&mut self, cx: &mut Context<Self>) -> bool {
        self.closing_window = true;
        let ready: bool = self.tabs.request_window_close();
        cx.notify();
        ready
    }

    fn finish_close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.closing_window && self.tabs.request_window_close() {
            if self.quit_after_close {
                cx.quit();
            } else {
                window.remove_window();
            }
        }
        cx.notify();
    }
}
