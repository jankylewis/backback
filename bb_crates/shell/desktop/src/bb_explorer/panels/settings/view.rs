//! The Settings window (Cmd+,): font family, size, tool window side and
//! Reset. Every change applies live to every window and is saved.

use bb_ui::Typography;
use gpui::{
    div, prelude::*, px, App, Bounds, Context, SharedString, Window, WindowBounds, WindowOptions,
};

use crate::color;

/// Monospace families offered when installed; Menlo is always there.
const CANDIDATES: [&str; 8] = [
    "Menlo",
    "SF Mono",
    "Monaco",
    "JetBrains Mono",
    "Fira Code",
    "Cascadia Code",
    "Source Code Pro",
    "Courier New",
];

pub(crate) struct SettingsWindow {
    families: Vec<SharedString>,
}

impl SettingsWindow {
    #[cfg(test)]
    pub(crate) fn with_families(families: Vec<SharedString>) -> Self {
        Self { families }
    }
}

pub(crate) fn open(cx: &mut App) {
    let installed: Vec<String> = cx.text_system().all_font_names();
    let families: Vec<SharedString> = CANDIDATES
        .iter()
        .filter(|name: &&&str| **name == "Menlo" || installed.iter().any(|f: &String| f == **name))
        .map(|name: &&str| SharedString::from(*name))
        .collect();
    let bounds: Bounds<gpui::Pixels> = Bounds::centered(None, gpui::size(px(440.0), px(520.0)), cx);
    let opened: gpui::Result<gpui::WindowHandle<SettingsWindow>> = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        },
        |_: &mut Window, cx: &mut App| {
            cx.new(|_: &mut Context<SettingsWindow>| SettingsWindow { families })
        },
    );
    if let Err(error) = opened {
        eprintln!("backback: cannot open Settings: {error}");
    }
}

fn button(
    id: SharedString,
    label: SharedString,
    on: bool,
    theme: bb_ui::Theme,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id.clone())
        .debug_selector(move || id.to_string())
        .px(px(10.0))
        .py(px(3.0))
        .rounded(px(4.0))
        .border_1()
        .border_color(color(if on { theme.accent } else { theme.border }))
        .cursor_pointer()
        .child(label)
}

fn heading(text: &'static str, theme: bb_ui::Theme) -> gpui::Div {
    div()
        .pt(px(8.0))
        .text_color(color(theme.text_dim))
        .child(text)
}

type Click = fn(&gpui::ClickEvent, &mut Window, &mut App);

fn change(cx: &mut App, edit: impl FnOnce(&mut Typography)) {
    super::bb_store::update_with(cx, edit);
}

fn fonts_row(families: &[SharedString], fonts: &Typography, theme: bb_ui::Theme) -> gpui::Div {
    let buttons: Vec<gpui::Stateful<gpui::Div>> = families
        .iter()
        .map(|family: &SharedString| {
            let chosen: SharedString = family.clone();
            let on: bool = fonts.family == family.as_ref();
            button(format!("font-{family}").into(), family.clone(), on, theme).on_click(
                move |_: &gpui::ClickEvent, _: &mut Window, cx: &mut App| {
                    let name: String = chosen.to_string();
                    change(cx, move |t: &mut Typography| t.family = name);
                },
            )
        })
        .collect();
    div().flex().flex_wrap().gap(px(6.0)).children(buttons)
}

fn size_row(fonts: &Typography, theme: bb_ui::Theme) -> gpui::Div {
    let down: Click = |_: &gpui::ClickEvent, _: &mut Window, cx: &mut App| {
        change(cx, |t: &mut Typography| t.step_base(false));
    };
    let up: Click = |_: &gpui::ClickEvent, _: &mut Window, cx: &mut App| {
        change(cx, |t: &mut Typography| t.step_base(true));
    };
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .child(button("size-down".into(), "-".into(), false, theme).on_click(down))
        .child(
            div()
                .debug_selector(|| "size-value".to_owned())
                .child(format!("{}", fonts.base)),
        )
        .child(button("size-up".into(), "+".into(), false, theme).on_click(up))
}

fn side_row(fonts: &Typography, theme: bb_ui::Theme) -> gpui::Div {
    let left: Click = |_: &gpui::ClickEvent, _: &mut Window, cx: &mut App| {
        change(cx, |t: &mut Typography| t.panel_right = false);
    };
    let right: Click = |_: &gpui::ClickEvent, _: &mut Window, cx: &mut App| {
        change(cx, |t: &mut Typography| t.panel_right = true);
    };
    div()
        .flex()
        .gap(px(6.0))
        .child(button("side-left".into(), "Left".into(), !fonts.panel_right, theme).on_click(left))
        .child(
            button(
                "side-right".into(),
                "Right".into(),
                fonts.panel_right,
                theme,
            )
            .on_click(right),
        )
}

impl Render for SettingsWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme: bb_ui::Theme = crate::theme::resolved_theme(None, Some(window.appearance()));
        let fonts: Typography = super::typography(cx);
        let reset: Click = |_: &gpui::ClickEvent, _: &mut Window, cx: &mut App| {
            change(cx, |t: &mut Typography| *t = Typography::default());
        };
        super::root(cx)
            .size_full()
            .p(px(16.0))
            .flex()
            .flex_col()
            .gap(px(6.0))
            .bg(color(theme.bg))
            .text_color(color(theme.text))
            .child(heading("Font", theme))
            .child(fonts_row(&self.families, &fonts, theme))
            .child(heading("Size", theme))
            .child(size_row(&fonts, theme))
            .child(heading("Tool window", theme))
            .child(side_row(&fonts, theme))
            .child(div().pt(px(12.0)).child(
                button("reset".into(), "Reset to default".into(), false, theme).on_click(reset),
            ))
    }
}
