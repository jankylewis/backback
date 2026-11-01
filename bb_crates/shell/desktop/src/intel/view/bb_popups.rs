//! Popups at the cursor (completion, code actions, the F2 box) and the
//! hover tooltip at the mouse. Each is deferred, so it paints above the
//! editor lines drawn after it.

use bb_intel::{ActionItem, CompletionEntry};
use bb_ui::bb_theme::Theme;
use gpui::{anchored, deferred, div, point, prelude::*, px, AnyElement, IntoElement};

use crate::color;
use crate::intel::model::popups::{ActionsPopup, CompletionPopup, HoverPopup, RenameInput};
use crate::intel::model::IntelModel;

/// Rows a list shows at once.
const ROWS: usize = 12;
/// What to draw below the cursor line, if any popup is open, `left` px from
/// the line row's left edge (the caret's x, gutter included).
pub(crate) fn at_cursor(model: &IntelModel, left: f32, theme: Theme) -> Option<AnyElement> {
    let body: AnyElement = if let Some(input) = &model.rename_input {
        rename_box(input, theme).into_any_element()
    } else if let Some(popup) = &model.completion {
        completion(popup, theme).into_any_element()
    } else if let Some(popup) = &model.actions {
        actions(popup, theme).into_any_element()
    } else {
        return None;
    };
    Some(
        div()
            .debug_selector(|| "cursor-popup".to_owned())
            .absolute()
            .top(px(22.0))
            .left(px(left))
            .child(deferred(anchored().child(body)).with_priority(1))
            .into_any_element(),
    )
}

fn frame(theme: Theme) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .min_w(px(260.0))
        .max_w(px(560.0))
        .bg(color(theme.overlay))
        .border_1()
        .border_color(color(theme.border))
        .text_color(color(theme.text))
}

fn row(text: String, detail: Option<String>, selected: bool, theme: Theme) -> gpui::Div {
    super::one_line(20.0)
        .px_2()
        .gap_2()
        .bg(color(if selected {
            theme.accent
        } else {
            theme.overlay
        }))
        .child(text)
        .when_some(detail, |el: gpui::Div, d: String| {
            el.child(div().text_color(color(theme.text_dim)).child(d))
        })
}

/// The window of rows to show around `selected`.
fn window_of(len: usize, selected: usize) -> std::ops::Range<usize> {
    let start: usize = selected.saturating_add(1).saturating_sub(ROWS);
    start..len.min(start.saturating_add(ROWS))
}

pub(crate) fn completion(popup: &CompletionPopup, theme: Theme) -> gpui::Div {
    let items: Vec<&CompletionEntry> = popup.visible();
    let shown: std::ops::Range<usize> = window_of(items.len(), popup.pick.selected);
    frame(theme).children(
        items
            .iter()
            .enumerate()
            .skip(shown.start)
            .take(shown.len())
            .map(|(index, item): (usize, &&CompletionEntry)| {
                row(
                    item.label.clone(),
                    item.detail.clone(),
                    index == popup.pick.selected,
                    theme,
                )
            }),
    )
}

fn actions(popup: &ActionsPopup, theme: Theme) -> gpui::Div {
    frame(theme).children(popup.items.iter().enumerate().map(
        |(index, item): (usize, &ActionItem)| {
            row(
                item.title.clone(),
                item.kind.clone(),
                index == popup.pick.selected,
                theme,
            )
        },
    ))
}

fn rename_box(input: &RenameInput, theme: Theme) -> gpui::Div {
    frame(theme)
        .p_1()
        .child(
            div()
                .text_color(color(theme.text_dim))
                .child("Rename to (Enter: preview, Esc: cancel)"),
        )
        .child(
            div()
                .px_1()
                .bg(color(theme.bg))
                .border_1()
                .border_color(color(theme.accent))
                .child(format!("{}│", input.text)),
        )
}

/// The hover tooltip at the mouse, if any.
pub(crate) fn hover(popup: Option<&HoverPopup>, theme: Theme) -> Option<AnyElement> {
    let popup: &HoverPopup = popup?;
    let body: gpui::Div = frame(theme).p_2().max_w(px(640.0)).children(
        popup
            .text
            .lines()
            .take(24)
            .map(|line: &str| div().child(line.to_owned())),
    );
    Some(
        deferred(
            anchored()
                .position(point(px(popup.x), px(popup.y + 16.0)))
                .child(body),
        )
        .with_priority(2)
        .into_any_element(),
    )
}
