//! The Debug panel's title row: what the session does, then the two
//! exception ticks of VS Code's Breakpoints view, `[x] All Exceptions` and
//! `[ ] User-Unhandled Exceptions`; a click flips one. Before the title,
//! Rider's debug toolbar: Stop, Resume, Step Over, Into, Out (BB-FEAT-082).

use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, AnyElement, ClickEvent, Context, Window};

use crate::intel::view::one_line;
use crate::run::bb_debug::session::control::Live;
use crate::run::bb_debug::ExceptionKind;
use crate::{color, ExplorerView};

/// The title, then the ticks at the right edge.
pub(super) fn title_row(
    view: &ExplorerView,
    title: String,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    one_line(22.0)
        .px_2()
        .gap_3()
        .text_color(color(theme.text_dim))
        .child(toolbar(view, theme, cx))
        .child(title)
        .child(div().flex_1())
        .child(tick(
            ExceptionKind::All,
            view.debug.exceptions.all,
            theme,
            cx,
        ))
        .child(tick(
            ExceptionKind::UserUnhandled,
            view.debug.exceptions.user_unhandled,
            theme,
            cx,
        ))
        .child(crate::intel::view::close_x(
            "debug-panel-close",
            theme,
            cx,
            hide,
        ))
        .into_any_element()
}

/// What a toolbar button does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tool {
    Stop,
    Resume,
    Over,
    Into,
    Out,
}

impl Tool {
    /// Id, glyph and tooltip, as Rider's debug toolbar.
    fn look(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Stop => ("debug-stop", "■", "Stop (Shift+F5)"),
            Self::Resume => ("debug-resume", "▶", "Resume to the next breakpoint (F5)"),
            Self::Over => ("debug-step-over", "↷", "Step Over (F10)"),
            Self::Into => ("debug-step-into", "↓", "Step Into (F11)"),
            Self::Out => ("debug-step-out", "↑", "Step Out (Shift+F11)"),
        }
    }

    fn run(self, view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>) {
        use crate::run::bb_debug::session::{control, StepKind};
        match self {
            Self::Stop => control::stop(view),
            Self::Resume => control::start_or_continue(view, window, cx),
            Self::Over => control::step(view, StepKind::Over),
            Self::Into => control::step(view, StepKind::Into),
            Self::Out => control::step(view, StepKind::Out),
        }
    }
}

/// Stop, then Resume and the steps, which only work while paused
/// (BB-FEAT-082); greyed while the program runs.
fn toolbar(view: &ExplorerView, theme: Theme, cx: &mut Context<ExplorerView>) -> AnyElement {
    let paused: bool = view.debug.live.as_ref().is_some_and(|live: &Live| {
        matches!(
            live.phase,
            crate::run::bb_debug::session::control::Phase::Stopped { .. }
        )
    });
    div()
        .flex()
        .items_center()
        .gap_1()
        .child(tool(Tool::Stop, true, theme.error, theme, cx))
        .children(
            [Tool::Resume, Tool::Over, Tool::Into, Tool::Out]
                .map(|kind: Tool| tool(kind, paused, theme.ok, theme, cx)),
        )
        .into_any_element()
}

/// One button; a click runs it, as its key does.
fn tool(
    kind: Tool,
    on: bool,
    ink: bb_ui::Rgb,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    let (id, glyph, tip): (&'static str, &'static str, &'static str) = kind.look();
    let button: gpui::Stateful<gpui::Div> = one_line(22.0)
        .id(id)
        .debug_selector(move || id.to_owned())
        .px_1()
        .rounded_sm()
        .tooltip(crate::tooltip::path(tip.to_owned(), theme))
        .child(glyph);
    if !on {
        return button.text_color(color(theme.text_dim)).into_any_element();
    }
    button
        .cursor_pointer()
        .text_color(color(ink))
        .hover(|el: gpui::StyleRefinement| el.bg(color(theme.overlay)))
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                kind.run(view, window, cx);
                cx.notify();
            },
        ))
        .into_any_element()
}

/// The ✕: hides the panel; the session goes on (BB-BUG-047).
pub(crate) fn hide(view: &mut ExplorerView, _: &mut Window, _: &mut Context<ExplorerView>) {
    if let Some(live) = view.debug.live.as_mut() {
        live.hidden = true;
    }
}

/// One tick; a click flips it.
fn tick(kind: ExceptionKind, on: bool, theme: Theme, cx: &mut Context<ExplorerView>) -> AnyElement {
    let mark: &str = if on { "[x]" } else { "[ ]" };
    one_line(22.0)
        .id(kind.id())
        .debug_selector(move || kind.id().to_owned())
        .gap_1()
        .cursor_pointer()
        .hover(|el: gpui::StyleRefinement| el.bg(color(theme.overlay)))
        .text_color(color(if on { theme.text } else { theme.text_dim }))
        .child(mark.to_owned())
        .child(kind.label().to_owned())
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &ClickEvent,
                  _: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                crate::run::bb_debug::toggle_exceptions(view, kind);
                cx.notify();
            },
        ))
        .into_any_element()
}
