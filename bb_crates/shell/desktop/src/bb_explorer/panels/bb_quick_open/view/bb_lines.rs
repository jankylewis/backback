//! The box's typed lines: the query, and Find in Files' "in" and "files"
//! fields. Only the field keys go to draws the caret (BB-BUG-057).

use gpui::{div, prelude::*, px};

use super::super::bb_model::{split_at, Field, FindFields};
use super::super::QuickOpen;
use crate::color;

type Theme = bb_ui::bb_theme::Theme;

/// The query line; for Find in Files, its three fields and a key hint.
pub(super) fn query_lines(quick: &QuickOpen, theme: Theme) -> gpui::Div {
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
