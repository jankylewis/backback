//! The terminal's title row: `TERMINAL`, one chip per shell when there is
//! more than one ("zsh 2 ✕": a click takes it, its ✕ ends it), `+` (split)
//! and the ✕ that hides the panel; the shells go on (BB-BUG-047).

use gpui::{div, prelude::*, AnyElement, Context, Window};

use crate::{color, ExplorerView};

pub(crate) fn title(
    view: &ExplorerView,
    theme: bb_ui::Theme,
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    let chips: Vec<AnyElement> = if view.terminal.panes.len() > 1 {
        (0..view.terminal.panes.len())
            .map(|index: usize| chip(view, index, theme, cx))
            .collect()
    } else {
        Vec::new()
    };
    crate::intel::view::one_line(crate::intel::view::TITLE_PX)
        .px_2()
        .gap_1()
        .text_color(color(theme.text_dim))
        .child(div().flex_1().child("TERMINAL"))
        .children(chips)
        .child(
            button("terminal-split", "+", theme)
                .on_click(cx.listener(
                    |view: &mut ExplorerView,
                     _: &gpui::ClickEvent,
                     window: &mut Window,
                     cx: &mut Context<ExplorerView>| {
                        super::on_split(view, &super::SplitTerminal, window, cx);
                    },
                ))
                .tooltip(crate::tooltip::path(
                    "Split Terminal (Cmd+\\)".to_owned(),
                    theme,
                )),
        )
        .child(crate::intel::view::close_x(
            "terminal-close",
            theme,
            cx,
            super::super::hide,
        ))
        .into_any_element()
}

/// Shell `index`: its name (the active one bright) and its ✕.
fn chip(
    view: &ExplorerView,
    index: usize,
    theme: bb_ui::Theme,
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    let number: u32 = view
        .terminal
        .panes
        .get(index)
        .map_or(0, |pane: &super::Pane| pane.number);
    let active: bool = index == view.terminal.active;
    div()
        .id(("terminal-chip", index))
        .debug_selector(move || format!("terminal-chip-{index}"))
        .flex()
        .items_center()
        .px_1()
        .rounded_sm()
        .cursor_pointer()
        .when(active, |el: gpui::Stateful<gpui::Div>| {
            el.bg(color(theme.overlay)).text_color(color(theme.text))
        })
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                view.terminal.active = index;
                super::super::show(view, window, cx);
            },
        ))
        .child(format!("zsh {number}"))
        .child(
            button(("terminal-kill", index), "✕", theme)
                .debug_selector(move || format!("terminal-kill-{index}"))
                .on_click(cx.listener(
                    move |view: &mut ExplorerView,
                          _: &gpui::ClickEvent,
                          window: &mut Window,
                          cx: &mut Context<ExplorerView>| {
                        cx.stop_propagation();
                        super::close(view, index, window, cx);
                    },
                )),
        )
        .into_any_element()
}

/// A small clickable glyph that lights up under the mouse.
fn button(
    id: impl Into<gpui::ElementId>,
    glyph: &'static str,
    theme: bb_ui::Theme,
) -> gpui::Stateful<gpui::Div> {
    let id: gpui::ElementId = id.into();
    let name: String = id.to_string();
    div()
        .id(id)
        .debug_selector(move || name.clone())
        .px_1()
        .cursor_pointer()
        .hover(move |el: gpui::StyleRefinement| {
            el.bg(color(theme.overlay)).text_color(color(theme.text))
        })
        .child(glyph)
}
