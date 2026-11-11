//! The box's typed lines: the query, and Find in Files' "in" and "files"
//! fields. Only the field keys go to draws the caret (BB-BUG-057). Find in
//! Files also shows Rider's scope buttons and its Cc, W and .* toggles.

use gpui::{div, prelude::*, px};

use gpui::Context;

use super::super::bb_model::{split_at, Field, FindFields, Scope, Toggle};
use super::super::QuickOpen;
use crate::{color, ExplorerView};

type Theme = bb_ui::bb_theme::Theme;

/// The query line; for Find in Files, its three fields and a key hint.
pub(super) fn query_lines(
    quick: &QuickOpen,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> gpui::Div {
    let (before, after): (&str, &str) = quick.split();
    let Some(fields): Option<&FindFields> = quick.fields() else {
        let title: String = format!("{}: ", quick.title());
        return div().child(line(&title, (before, after), true, None, theme));
    };
    let on: Field = fields.field;
    div()
        .flex()
        .flex_col()
        .child(line(
            "Find in Files: ",
            (before, after),
            on == Field::Text,
            None,
            theme,
        ))
        .child(buttons(fields, theme, cx))
        .child(
            line(
                "in: ",
                split_at(&fields.in_text, fields.in_caret),
                on == Field::In,
                Some("type to narrow the places below"),
                theme,
            )
            .debug_selector(|| "quick-open-in".to_owned()),
        )
        .child(
            line(
                "files: ",
                split_at(&fields.files, fields.files_caret),
                on == Field::Files,
                Some("e.g. *.cs, !*.md, !Tests"),
                theme,
            )
            .debug_selector(|| "quick-open-files".to_owned()),
        )
        .child(
            div()
                .pt(px(2.0))
                .text_color(color(theme.text_dim))
                .child("Tab: next field · Up/Down: place · Enter: search"),
        )
}

/// One field: its title, its text with the caret when keys go to it, and a
/// dim hint while it is empty.
fn line(
    title: &str,
    (before, after): (&str, &str),
    active: bool,
    hint: Option<&'static str>,
    theme: Theme,
) -> gpui::Div {
    let empty: bool = before.is_empty() && after.is_empty();
    div()
        .py(px(2.0))
        .flex()
        .items_center()
        .child(format!("{title}{before}"))
        .when(active, |d: gpui::Div| {
            d.child(
                div()
                    .w(px(2.0))
                    .h(px(16.0))
                    .bg(color(theme.text))
                    .debug_selector(|| "quick-open-caret".to_owned()),
            )
        })
        .child(after.to_owned())
        .when_some(hint.filter(|_: &&str| empty), |d: gpui::Div, hint: &str| {
            d.child(
                div()
                    .pl(px(8.0))
                    .text_color(color(theme.text_dim))
                    .child(hint),
            )
        })
}

/// Rider's scope buttons, then its toggles; the ones in use are filled.
fn buttons(fields: &FindFields, theme: Theme, cx: &mut Context<ExplorerView>) -> gpui::Div {
    let scopes: [(&'static str, Scope); 3] = [
        ("Solution", Scope::Solution),
        ("Project", Scope::Project),
        ("Directory", Scope::Directory),
    ];
    let toggles: [(&'static str, Toggle, bool); 3] = [
        ("Cc", Toggle::Case, fields.options.case),
        ("W", Toggle::Word, fields.options.word),
        (".*", Toggle::Regex, fields.options.regex),
    ];
    div()
        .py(px(2.0))
        .flex()
        .gap(px(6.0))
        .children(scopes.map(|(name, scope): (&'static str, Scope)| {
            button(name, fields.scope == scope, theme, cx).on_click(cx.listener(
                move |view: &mut ExplorerView,
                      _: &gpui::ClickEvent,
                      _: &mut gpui::Window,
                      cx: &mut Context<ExplorerView>| {
                    if let Some(quick) = view.quick_open.as_mut() {
                        quick.set_scope(scope);
                    }
                    cx.notify();
                },
            ))
        }))
        .child(div().flex_1())
        .children(
            toggles.map(|(name, which, on): (&'static str, Toggle, bool)| {
                button(name, on, theme, cx).on_click(cx.listener(
                    move |view: &mut ExplorerView,
                          _: &gpui::ClickEvent,
                          _: &mut gpui::Window,
                          cx: &mut Context<ExplorerView>| {
                        if let Some(quick) = view.quick_open.as_mut() {
                            quick.toggle(which);
                        }
                        cx.notify();
                    },
                ))
            }),
        )
}

fn button(
    name: &'static str,
    on: bool,
    theme: Theme,
    _: &mut Context<ExplorerView>,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(name)
        .debug_selector(move || format!("find-button-{name}"))
        .px(px(6.0))
        .rounded(px(3.0))
        .border_1()
        .border_color(color(if on { theme.accent } else { theme.border }))
        .when(on, |d: gpui::Stateful<gpui::Div>| {
            d.bg(color(theme.surface))
        })
        .cursor_pointer()
        .child(name)
}
