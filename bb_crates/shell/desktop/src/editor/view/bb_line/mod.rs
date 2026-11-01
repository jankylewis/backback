use crate::intel::view::marks::Ink;
use crate::{color, ExplorerView};
use bb_syntax::HighlightKind;
use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, Context, IntoElement, StyledText, TextRun, Window};
mod bb_runs;
mod caret;
mod gutter;
pub(super) use bb_runs::{styled_line, Run};
pub(crate) use caret::{Columns, GUTTER_W};
use gutter::gutter_cell;
pub(super) use gutter::TestGutter;
#[cfg(test)]
mod tests;

/// What a line row needs besides its text.
pub(super) struct LineRow {
    pub(super) index: usize,
    pub(super) current: bool,
    /// The worst diagnostic on the line, for the gutter dot.
    pub(super) gutter: Option<Ink>,
    pub(super) test: Option<TestGutter>,
    /// A popup anchored below the cursor.
    pub(super) overlay: Option<gpui::AnyElement>,
    /// On the cursor line: the caret's x from the text's left edge.
    pub(super) caret: Option<f32>,
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
        current,
        gutter,
        test,
        overlay,
        caret,
    }: LineRow = row;
    row_frame(index, current, theme, cx)
        .child(gutter_cell(index, gutter, test, theme, cx))
        .child(line_text(index, runs, theme))
        .when_some(caret, |el: gpui::Stateful<gpui::Div>, x: f32| {
            el.child(caret_quad(x, theme))
        })
        .children(overlay)
}

/// The row box: height, font, line background and the mouse handlers.
fn row_frame(
    index: usize,
    current: bool,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> gpui::Stateful<gpui::Div> {
    div()
        .relative()
        .id(index)
        .debug_selector(|| format!("editor-line-{index}"))
        .h(px(22.0))
        .flex()
        .items_center()
        .font_family("Menlo")
        .text_size(px(caret::FONT_SIZE))
        .text_color(color(theme.text))
        .bg(color(if current { theme.surface } else { theme.bg }))
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(
                move |view: &mut ExplorerView,
                      event: &gpui::MouseDownEvent,
                      window: &mut Window,
                      cx: &mut Context<ExplorerView>| {
                    mouse_down(index, view, event, window, cx);
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
fn line_text(index: usize, runs: &[Run], theme: Theme) -> impl IntoElement {
    let text: String = runs.iter().map(|run: &Run| run.text.as_str()).collect();
    let styles: Vec<TextRun> = runs.iter().map(|run: &Run| text_run(run, theme)).collect();
    div()
        .flex_none()
        .whitespace_nowrap()
        .debug_selector(move || format!("editor-text-{index}"))
        .child(StyledText::new(text).with_runs(styles))
}

fn text_run(run: &Run, theme: Theme) -> TextRun {
    let ink: bb_ui::Rgb = match run.kind {
        HighlightKind::Keyword => theme.keyword,
        HighlightKind::String => theme.string,
        HighlightKind::Comment => theme.comment,
        HighlightKind::Type => theme.ty,
        HighlightKind::Plain => theme.text,
    };
    TextRun {
        len: run.text.len(),
        font: gpui::font("Menlo"),
        color: color(ink).into(),
        background_color: run.selected.then(|| color(theme.accent).into()),
        underline: run.mark.map(|s: Ink| gpui::UnderlineStyle {
            thickness: px(1.0),
            color: Some(color(severity_ink(s, theme)).into()),
            wavy: true,
        }),
        strikethrough: None,
    }
}

/// The caret: a thin quad painted over the text, never a char in it.
fn caret_quad(x: f32, theme: Theme) -> impl IntoElement {
    div()
        .absolute()
        .top(px(3.0))
        .left(px(GUTTER_W + x))
        .w(px(caret::CARET_W))
        .h(px(16.0))
        .bg(color(theme.text))
        .debug_selector(|| "editor-caret".to_owned())
}

fn mouse_down(
    index: usize,
    view: &mut ExplorerView,
    event: &gpui::MouseDownEvent,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    super::super::input::pointer_at(
        view,
        index,
        f32::from(event.position.x),
        event.modifiers.shift,
        window,
        cx,
    );
}

fn mouse_move(
    index: usize,
    view: &mut ExplorerView,
    event: &gpui::MouseMoveEvent,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    if event.dragging() {
        super::super::input::pointer_at(view, index, f32::from(event.position.x), true, window, cx);
        return;
    }
    let x: f32 = f32::from(event.position.x);
    let at: Option<bb_text::CharIdx> = super::super::input::char_at(view, index, x, window);
    let position: (f32, f32) = (x, f32::from(event.position.y));
    crate::intel::glue::hover_move(view, at, position, window, cx);
}
