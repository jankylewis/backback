use bb_ui::theme::DARK;
use gpui::{div, prelude::*, AnyView, App, Context, IntoElement, Render, Window};

use super::color;

struct PathTooltip {
    path: String,
}

impl Render for PathTooltip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .p_2()
            .bg(color(DARK.overlay))
            .border_1()
            .border_color(color(DARK.border))
            .text_color(color(DARK.text))
            .child(self.path.clone())
    }
}

pub(crate) fn path(path: String) -> impl Fn(&mut Window, &mut App) -> AnyView {
    move |_: &mut Window, cx: &mut App| {
        cx.new(|_: &mut Context<PathTooltip>| PathTooltip { path: path.clone() })
            .into()
    }
}
