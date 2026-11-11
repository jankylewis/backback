use crate::intel::view::marks::Ink;
use crate::{color, ExplorerView};
use bb_syntax::HighlightKind;
use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, Context, IntoElement, StyledText, TextRun, Window};
mod bb_runs;
mod caret;
mod gutter;
#[cfg(test)]
use bb_runs::styled_line;
pub(super) use bb_runs::{styled_part, Run};
pub(crate) use caret::{rendered, Columns, CARET_W};
use gutter::gutter_cell;
pub(super) use gutter::TestGutter;
#[cfg(test)]
mod tests;

/// What a line row needs besides its text.
pub(super) struct LineRow {
    pub(super) index: usize,
    /// The screen row (the line unless word wrap is on) and, on a wrapped
    /// line's later row (no gutter), its indent in px.
    pub(super) at: (usize, Option<f32>),
    pub(super) current: bool,
    /// The worst diagnostic on the line, for the gutter dot.
    pub(super) gutter: Option<Ink>,
    /// A breakpoint on this line (BBEOD17).
    pub(super) breakpoint: bool,
    /// The debugger is paused on this line.
    pub(super) stopped: bool,
    pub(super) test: Option<TestGutter>,
    /// A popup anchored below the cursor.
    pub(super) overlay: Option<gpui::AnyElement>,
    /// On the cursor line: the caret's x from the text's left edge.
    pub(super) caret: Option<f32>,
    /// Pixels the text has moved left (BB-BUG-013).
    pub(super) scroll_x: f32,
    /// The bracket match's boxes and guide on this row.
    pub(super) braces: Vec<gpui::Div>,
}

impl LineRow {
    /// A row with no tab behind it (nothing loaded yet).
    pub(super) fn blank(index: usize) -> LineRow {
        LineRow {
            index,
            at: (index, None),
            current: false,
            gutter: None,
            breakpoint: false,
            stopped: false,
            test: None,
            overlay: None,
            caret: None,
            scroll_x: 0.0,
            braces: Vec::new(),
        }
    }
}

fn severity_ink(ink: Ink, theme: Theme) -> bb_ui::Rgb {
    match ink {
        Ink::Error => theme.error,
        Ink::Warning => theme.warn,
        Ink::Faint => theme.text_dim,
    }
}

pub(super) fn line_row(
    row: LineRow,
    runs: &[Run],
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    let LineRow {
        index,
        at: (row, continued),
        current,
        gutter,
        breakpoint,
        stopped,
        test,
        overlay,
        caret,
        scroll_x,
        braces,
    }: LineRow = row;
    let fonts: bb_ui::Typography = crate::settings::typography(cx);
    let gutter: gpui::Div = match continued {
        None => gutter_cell(index, (gutter, breakpoint, stopped), test, theme, cx),
        Some(_) => div().w(px(fonts.gutter_width())).flex_none(),
    };
    row_frame(row, (current, stopped), (theme, fonts.line_height()), cx)
        .child(gutter)
        .child(super::pane::bb_hscroll::text_viewport(
            scroll_x,
            line_text(index, runs, (theme, &fonts)).pl(px(continued.unwrap_or(0.0))),
        ))
        .when_some(
            caret.map(|x: f32| x - scroll_x).filter(|x: &f32| *x >= 0.0),
            |el: gpui::Stateful<gpui::Div>, x: f32| el.child(caret_quad(x, theme, &fonts)),
        )
        .children(braces)
        .children(overlay)
}

/// The row box: height, font, line background and the mouse handlers.
fn row_frame(
    index: usize,
    (current, stopped): (bool, bool),
    (theme, height): (Theme, f32),
    cx: &mut Context<ExplorerView>,
) -> gpui::Stateful<gpui::Div> {
    // The whole pane width: a click right of the text still lands on the
    // row (BB-BUG-022), and the current line's band spans the pane.
    div()
        .relative()
        .w_full()
        .id(index)
        .debug_selector(|| format!("editor-line-{index}"))
        .h(px(height))
        .flex()
        .items_center()
        .text_color(color(theme.text))
        .bg(color(if current { theme.surface } else { theme.bg }))
        .when(stopped, |row: gpui::Stateful<gpui::Div>| {
            row.bg(gpui::Rgba {
                a: 0.28,
                ..color(theme.warn)
            })
        })
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(
                move |view: &mut ExplorerView,
                      event: &gpui::MouseDownEvent,
                      window: &mut Window,
                      cx: &mut Context<ExplorerView>| {
                    crate::editor::link::press(index, view, event, window, cx);
                },
            ),
        )
        .on_mouse_move(cx.listener(
            move |view: &mut ExplorerView,
                  event: &gpui::MouseMoveEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                mouse_move(index, view, event, window, cx);
            },
        ))
}

/// The file's chars as ONE shaped line, so the glyphs sit exactly where
/// `Columns` measures them; colour, selection and squiggles are run styles.
fn line_text(index: usize, runs: &[Run], (theme, fonts): (Theme, &bb_ui::Typography)) -> gpui::Div {
    let text: String = runs.iter().map(|run: &Run| run.text.as_str()).collect();
    let styles: Vec<TextRun> = runs
        .iter()
        .map(|run: &Run| text_run(run, theme, &fonts.family))
        .collect();
    div()
        .flex_none()
        .whitespace_nowrap()
        .debug_selector(move || format!("editor-text-{index}"))
        .child(StyledText::new(text).with_runs(styles))
}

pub(super) fn text_run(run: &Run, theme: Theme, family: &str) -> TextRun {
    let ink: bb_ui::Rgb = match (run.selected, run.link, run.kind) {
        (true, _, _) | (false, false, HighlightKind::Plain) => theme.text, // BB-BUG-054
        (false, true, _) => theme.accent,
        (false, false, HighlightKind::Keyword) => theme.keyword,
        (false, false, HighlightKind::String) => theme.string,
        (false, false, HighlightKind::Comment) => theme.comment,
        (false, false, HighlightKind::Type) => theme.ty,
    };
    let link_line: Option<gpui::UnderlineStyle> = run.link.then(|| gpui::UnderlineStyle {
        thickness: px(1.0),
        color: Some(color(theme.accent).into()),
        wavy: false,
    });
    TextRun {
        len: run.text.len(),
        font: gpui::font(family.to_owned()),
        color: color(ink).into(),
        background_color: run.selected.then(|| color(theme.selection).into()),
        underline: link_line.or_else(|| {
            run.mark.map(|s: Ink| gpui::UnderlineStyle {
                thickness: px(1.0),
                color: Some(color(severity_ink(s, theme)).into()),
                wavy: true,
            })
        }),
        strikethrough: None,
    }
}

/// The caret: a thin quad painted over the text, never a char in it.
/// The caret spans the text height (size + 3), centred in the row.
fn caret_quad(x: f32, theme: Theme, fonts: &bb_ui::Typography) -> impl IntoElement {
    let height: f32 = fonts.size() + 3.0;
    div()
        .absolute()
        .top(px((fonts.line_height() - height) / 2.0))
        .left(px(fonts.gutter_width() + x))
        .w(px(caret::CARET_W))
        .h(px(height))
        .bg(color(theme.text))
        .debug_selector(|| "editor-caret".to_owned())
}

fn mouse_move(
    index: usize,
    view: &mut ExplorerView,
    event: &gpui::MouseMoveEvent,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    if event.dragging() {
        // A drag that began on a divider, a scroll bar or a tab is not a
        // text selection (BB-BUG-063).
        if view.lines.text_press {
            super::super::input::pointer_at(
                view,
                index,
                f32::from(event.position.x),
                true,
                window,
                cx,
            );
        }
        return;
    }
    view.lines.text_press = false;
    let x: f32 = f32::from(event.position.x);
    let fonts: bb_ui::Typography = crate::settings::typography(cx);
    let at: Option<bb_text::CharIdx> =
        super::super::input::char_at(view, index, x, (window, &fonts));
    let position: (f32, f32) = (x, f32::from(event.position.y));
    crate::editor::link::link_move(view, at, event.modifiers.platform);
    crate::intel::glue::hover_move(view, at, position, window, cx);
}
