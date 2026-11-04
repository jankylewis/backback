//! The Debug panel under the editor while a session lives: the program's
//! output while it builds or runs; when paused, Call Stack next to
//! Variables (as in VS Code), under the exception the program stopped on,
//! if any. A frame with source opens its call site on a click; frames
//! without source (the framework) are listed, greyed. The title row carries
//! the two exception ticks (`ticks`).

use std::path::Path;

use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, AnyElement, ClickEvent, Context, Window};

use super::notes::select_frame;
use super::ticks::title_row;
use crate::intel::view::one_line;
use crate::run::bb_debug::session::control::{Live, Phase};
use crate::run::bb_debug::session::{Frame, Var};
use crate::{color, ExplorerView};

/// Output lines shown while the program runs.
const TAIL: usize = 12;

/// The panel, if a session lives.
pub(crate) fn panel(view: &ExplorerView, cx: &mut Context<ExplorerView>) -> Option<AnyElement> {
    let live: &Live = view.debug.live.as_ref()?;
    let theme: Theme = view.theme;
    let body: AnyElement = match &live.phase {
        Phase::Building | Phase::Running => output(live, theme),
        Phase::Stopped {
            frames,
            selected,
            vars,
            exception,
        } => paused(
            view,
            (frames.as_slice(), *selected, vars.as_slice()),
            exception.as_deref(),
            theme,
            cx,
        ),
    };
    Some(
        div()
            .id("debug-panel")
            .debug_selector(|| "debug-panel".to_owned())
            .h(px(view.lines.bottom_px))
            .flex_none()
            .flex()
            .flex_col()
            .bg(color(theme.surface))
            .text_color(color(theme.text))
            .child(crate::editor::bottom_handle(theme, cx))
            .child(title_row(view, title(live), theme, cx))
            .child(body)
            .into_any_element(),
    )
}

/// `Debug: <project> building | running | paused [on an exception]`.
fn title(live: &Live) -> String {
    match &live.phase {
        Phase::Building => format!("Debug: building {}", live.label),
        Phase::Running => format!("Debug: {} running", live.label),
        Phase::Stopped {
            exception: Some(_), ..
        } => format!("Debug: {} paused on an exception", live.label),
        Phase::Stopped {
            exception: None, ..
        } => format!("Debug: {} paused", live.label),
    }
}

/// Paused: the exception, if any, above Call Stack next to Variables.
fn paused(
    view: &ExplorerView,
    (frames, selected, vars): (&[Frame], usize, &[Var]),
    exception: Option<&str>,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_h(px(0.0))
        .children(exception.map(|what: &str| exception_row(what, theme)))
        .child(
            div()
                .flex()
                .flex_1()
                .min_h(px(0.0))
                .child(stack(frames, selected, &view.root, theme, cx))
                .child(variables(vars, theme)),
        )
        .into_any_element()
}

fn output(live: &Live, theme: Theme) -> AnyElement {
    let lines: Vec<&str> = live.output.lines().collect();
    let tail: &[&str] = lines
        .get(lines.len().saturating_sub(TAIL)..)
        .unwrap_or_default();
    div()
        .px_2()
        .flex()
        .flex_col()
        .text_color(color(theme.text_dim))
        .children(
            tail.iter()
                .map(|line: &&str| one_line(18.0).child((*line).to_owned())),
        )
        .into_any_element()
}

fn column(heading: &str, theme: Theme) -> gpui::Div {
    div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .px_2()
        .child(
            one_line(20.0)
                .text_color(color(theme.text_dim))
                .child(heading.to_owned()),
        )
}

fn stack(
    frames: &[Frame],
    selected: usize,
    root: &Path,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> gpui::Div {
    let rows: Vec<AnyElement> = frames
        .iter()
        .enumerate()
        .map(|(index, frame): (usize, &Frame)| {
            frame_row(index, frame, (index == selected, root), theme, cx)
        })
        .collect();
    column("CALL STACK", theme).children(rows)
}

fn frame_row(
    index: usize,
    frame: &Frame,
    (picked, root): (bool, &Path),
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    let place: String = frame.path.as_deref().map_or_else(
        || String::from("no source"),
        |path: &Path| {
            let short: &Path = path.strip_prefix(root).unwrap_or(path);
            format!("{}:{}", short.display(), frame.line.saturating_add(1))
        },
    );
    let row: gpui::Stateful<gpui::Div> = one_line(20.0)
        .id(("frame", index))
        .debug_selector(move || format!("debug-frame-{index}"))
        .gap_2()
        .when(picked, |row: gpui::Stateful<gpui::Div>| {
            row.bg(color(theme.overlay))
        })
        .child(frame.name.clone())
        .child(div().text_color(color(theme.text_dim)).child(place));
    if frame.path.is_none() {
        return row.text_color(color(theme.text_dim)).into_any_element();
    }
    row.cursor_pointer()
        .hover(|el: gpui::StyleRefinement| el.bg(color(theme.overlay)))
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                select_frame(view, index, window, cx);
                cx.notify();
            },
        ))
        .into_any_element()
}

fn variables(vars: &[Var], theme: Theme) -> gpui::Div {
    column("VARIABLES", theme).children(vars.iter().map(|var: &Var| {
        one_line(20.0).gap_2().child(var.name.clone()).child(
            div()
                .text_color(color(theme.string))
                .child(var.value.clone()),
        )
    }))
}

/// The exception paused on, above the stack, in the error colour.
fn exception_row(what: &str, theme: Theme) -> AnyElement {
    one_line(20.0)
        .id("debug-exception")
        .debug_selector(|| "debug-exception".to_owned())
        .px_2()
        .text_color(color(theme.error))
        .child(what.to_owned())
        .into_any_element()
}
