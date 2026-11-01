//! The Go to File box: typed query, best matches, Enter opens, Esc closes.

use gpui::{deferred, div, prelude::*, px, AnyElement, Context, KeyDownEvent, Window};

use super::QuickOpen;
use crate::{color, ExplorerView};

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct GoToFile;

pub(crate) fn on_go_to_file(
    view: &mut ExplorerView,
    _: &GoToFile,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    show(view, window, cx);
}

pub(crate) fn show(view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>) {
    view.quick_open = Some(QuickOpen::new(&view.root));
    window.focus(&view.quick_focus, cx);
    cx.notify();
}

fn key(
    view: &mut ExplorerView,
    event: &KeyDownEvent,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    cx.stop_propagation();
    let key: &str = event.keystroke.key.as_str();
    if key == "escape" || key == "enter" {
        let chosen: Option<std::path::PathBuf> = view
            .quick_open
            .take()
            .and_then(|quick: QuickOpen| (key == "enter").then(|| quick.chosen()).flatten());
        if let Some(path) = chosen {
            view.open_file(&path, window, cx);
        }
        window.focus(&view.editor_focus, cx);
        cx.notify();
        return;
    }
    let Some(quick): Option<&mut QuickOpen> = view.quick_open.as_mut() else {
        return;
    };
    match key {
        "backspace" => quick.backspace(),
        "up" => quick.step(false),
        "down" => quick.step(true),
        _ => {
            if let (Some(text), false) = (
                event.keystroke.key_char.as_deref(),
                event.keystroke.modifiers.platform,
            ) {
                quick.type_text(text);
            }
        }
    }
    cx.notify();
}

fn rows(quick: &QuickOpen, theme: bb_ui::bb_theme::Theme) -> Vec<gpui::Div> {
    quick
        .matches()
        .into_iter()
        .enumerate()
        .map(|(index, path): (usize, &std::path::Path)| {
            div()
                .px(px(8.0))
                .py(px(2.0))
                .rounded(px(3.0))
                .when(index == quick.selected, |row: gpui::Div| {
                    row.bg(color(theme.surface))
                })
                .child(path.display().to_string())
        })
        .collect()
}

pub(crate) fn overlay(view: &ExplorerView, cx: &mut Context<ExplorerView>) -> Option<AnyElement> {
    let quick: &QuickOpen = view.quick_open.as_ref()?;
    let theme: bb_ui::bb_theme::Theme = view.theme;
    let rows: Vec<gpui::Div> = rows(quick, theme);
    let query: String = format!("Go to File: {}", quick.query);
    let panel: gpui::Stateful<gpui::Div> = div()
        .id("quick-open")
        .track_focus(&view.quick_focus)
        .on_key_down(cx.listener(key))
        .w(px(560.0))
        .p(px(6.0))
        .flex()
        .flex_col()
        .bg(color(theme.overlay))
        .border_1()
        .border_color(color(theme.border))
        .rounded(px(6.0))
        .child(
            div()
                .px(px(8.0))
                .py(px(4.0))
                .debug_selector(|| "quick-open-query".to_owned())
                .child(query),
        )
        .children(rows);
    Some(
        deferred(
            div()
                .absolute()
                .top(px(40.0))
                .left(px(0.0))
                .w_full()
                .flex()
                .justify_center()
                .child(panel),
        )
        .with_priority(2)
        .into_any_element(),
    )
}
