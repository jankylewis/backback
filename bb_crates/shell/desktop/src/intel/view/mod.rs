//! Drawing code intelligence: popups, panels, squiggles.

pub(crate) mod bb_popups;
pub(crate) mod marks;
pub(crate) mod panels;

use std::time::Instant;

use gpui::{div, prelude::*, px};

use crate::intel::model::{IntelModel, TraceRow, Undrawn};

/// One row of a list or panel: a fixed height, one line, cut with an
/// ellipsis. Every list row uses it, so long text never wraps over the next
/// row or widens the window.
pub(crate) fn one_line(height: f32) -> gpui::Div {
    div()
        .min_h(px(height))
        .flex_none()
        .flex()
        .items_center()
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
}

/// Height of a panel's title row.
pub(crate) const TITLE_PX: f32 = 22.0;

/// The ✕ at the right of a panel's title row: a click closes the panel
/// (BB-BUG-047).
pub(crate) fn close_x(
    id: &'static str,
    theme: bb_ui::bb_theme::Theme,
    cx: &mut gpui::Context<crate::ExplorerView>,
    close: fn(&mut crate::ExplorerView, &mut gpui::Window, &mut gpui::Context<crate::ExplorerView>),
) -> gpui::AnyElement {
    div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .px_1()
        .cursor_pointer()
        .text_color(crate::color(theme.text_dim))
        .hover(|el: gpui::StyleRefinement| {
            el.bg(crate::color(theme.overlay))
                .text_color(crate::color(theme.text))
        })
        .child("✕")
        .on_click(cx.listener(
            move |view: &mut crate::ExplorerView,
                  _: &gpui::ClickEvent,
                  window: &mut gpui::Window,
                  cx: &mut gpui::Context<crate::ExplorerView>| {
                cx.stop_propagation();
                close(view, window, cx);
                cx.notify();
            },
        ))
        .into_any_element()
}

/// Count a frame. A completion answer applied since the last frame is drawn
/// in this one: record how many frames and how long it waited.
pub(crate) fn note_frame(model: &mut IntelModel) {
    model.frame = model.frame.saturating_add(1);
    if model.completion.is_none() {
        model.undrawn = None;
        return;
    }
    let Some(undrawn): Option<Undrawn> = model.undrawn.take() else {
        return;
    };
    let row: TraceRow = TraceRow {
        queued: undrawn.stages.queued(),
        server: undrawn.stages.server(),
        hop: undrawn
            .applied
            .saturating_duration_since(undrawn.stages.answered),
        frames: model.frame.saturating_sub(undrawn.frame).saturating_sub(1),
        to_paint: Instant::now().saturating_duration_since(undrawn.stages.answered),
    };
    eprintln!(
        "intel_trace kind=completion queued_us={} server_us={} hop_us={} frames={} to_paint_us={}",
        row.queued.as_micros(),
        row.server.as_micros(),
        row.hop.as_micros(),
        row.frames,
        row.to_paint.as_micros(),
    );
    model.trace.push(row);
}

#[cfg(test)]
mod tests {
    use gpui::Styled;

    #[test]
    fn a_list_row_is_one_clipped_line_with_an_ellipsis() {
        let mut row: gpui::Div = super::one_line(20.0);
        let style: &mut gpui::StyleRefinement = row.style();
        assert_eq!(style.text.white_space, Some(gpui::WhiteSpace::Nowrap));
        assert!(style.text.text_overflow.is_some());
        assert_eq!(style.overflow.y, Some(gpui::Overflow::Hidden));
    }

    #[test]
    fn every_fixed_height_row_in_the_intel_view_is_a_one_line_row() {
        let dir: std::path::PathBuf =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/intel/view");
        let mut offenders: Vec<String> = Vec::new();
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path: std::path::PathBuf = entry.unwrap().path();
            let text: String = std::fs::read_to_string(&path).unwrap();
            let body: &str = text.split("#[cfg(test)]").next().unwrap_or_default();
            let rows: usize =
                body.matches(".h(px(20.0))").count() + body.matches(".h(px(22.0))").count();
            if rows > 0 {
                offenders.push(path.display().to_string());
            }
        }
        assert_eq!(offenders, Vec::<String>::new());
    }
}

#[cfg(test)]
mod panels_tests;
