use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, AnyView, App, Context, IntoElement, Render, Window};

use super::color;

struct PathTooltip {
    path: String,
    theme: Theme,
}

impl Render for PathTooltip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .p_2()
            .bg(color(self.theme.overlay))
            .border_1()
            .border_color(color(self.theme.border))
            .text_color(color(self.theme.text))
            .child(self.path.clone())
    }
}

pub(crate) fn path(path: String, theme: Theme) -> impl Fn(&mut Window, &mut App) -> AnyView {
    move |_: &mut Window, cx: &mut App| {
        cx.new(|_: &mut Context<PathTooltip>| PathTooltip {
            path: path.clone(),
            theme,
        })
        .into()
    }
}
