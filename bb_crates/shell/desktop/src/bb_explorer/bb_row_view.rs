use std::path::PathBuf;

use bb_explorer::Entry;
use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, Context, IntoElement, Window};

use crate::icons::chevron;
use crate::{color, icons, tooltip, ExplorerView};

#[derive(Clone, Copy)]
pub(crate) struct RowState {
    pub(crate) selected: bool,
    pub(crate) panel_width: f32,
    pub(crate) theme: Theme,
}

struct RowAction {
    index: usize,
    path: PathBuf,
    expandable: bool,
}

pub(crate) fn row(
    index: usize,
    depth: usize,
    entry: &Entry,
    state: RowState,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    let action: RowAction = RowAction {
        index,
        path: entry.path().to_path_buf(),
        expandable: entry.expandable(),
    };
    let tooltip_path: String = action.path.display().to_string();
    let name: String = entry.name().to_owned();
    let text_color: gpui::Rgba = row_text_color(entry, state.theme);
    let layout: RowLayout = RowLayout::new(depth, state.panel_width, f32::MAX);
    let border: gpui::Rgba = row_border_color(state);
    div()
        .id(index)
        .pl_2()
        .h(px(30.0))
        .flex()
        .items_center()
        .gap_0()
        .text_color(text_color)
        .border_1()
        .border_color(border)
        .hover(move |style: gpui::StyleRefinement| style.bg(color(state.theme.overlay)))
        .tooltip(tooltip::path(tooltip_path, state.theme))
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                activate(view, &action, window, cx);
            },
        ))
        .children(
            layout
                .guide_offsets
                .iter()
                .map(|_: &f32| indent_guide(state.theme)),
        )
        .child(div().w(px(16.0)).flex_shrink_0().child(chevron(entry)))
        .child(icons::icon(entry, color(icons::tint(entry, &state.theme))))
        .child(name_box(name, layout.name_width))
}

fn row_text_color(entry: &Entry, theme: Theme) -> gpui::Rgba {
    color(if entry.dimmed() {
        theme.text_dim
    } else {
        theme.text
    })
}

fn row_border_color(state: RowState) -> gpui::Rgba {
    color(if state.selected {
        state.theme.accent
    } else {
        state.theme.surface
    })
}

fn activate(
    view: &mut ExplorerView,
    action: &RowAction,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    view.selected = action.index;
    if action.expandable {
        if let Err(error) = view.tree.toggle(&action.path) {
            view.report(&error);
        }
    } else {
        view.open_file(&action.path, window, cx);
    }
    cx.notify();
}

fn name_box(name: String, width: f32) -> impl IntoElement {
    div()
        .ml_1()
        .w(px(width))
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
        .child(name)
}

fn indent_guide(theme: Theme) -> impl IntoElement {
    div()
        .w(px(16.0))
        .flex_shrink_0()
        .h_full()
        .border_l_1()
        .border_color(color(theme.border))
}

struct RowLayout {
    guide_offsets: Vec<f32>,
    name_width: f32,
}

impl RowLayout {
    fn new(depth: usize, panel_width: f32, desired_name_width: f32) -> Self {
        let guide_offsets: Vec<f32> = (0..depth)
            .map(|index: usize| 8.0 + 16.0 * f32::from(u16::try_from(index).unwrap_or(u16::MAX)))
            .collect();
        let name_width: f32 =
            (panel_width - 56.0 - 16.0 * f32::from(u16::try_from(depth).unwrap_or(u16::MAX)))
                .max(0.0)
                .min(desired_name_width);
        Self {
            guide_offsets,
            name_width,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RowLayout;

    #[test]
    fn long_rows_keep_one_fixed_guide_per_depth_through_twelve() {
        for depth in 0..=12 {
            let layout: RowLayout = RowLayout::new(depth, 300.0, 600.0);
            assert_eq!(layout.guide_offsets.len(), depth);
            for (index, offset) in layout.guide_offsets.iter().enumerate() {
                assert_eq!(
                    *offset,
                    8.0 + 16.0 * f32::from(u16::try_from(index).unwrap_or(u16::MAX))
                );
            }
            assert!(layout.name_width >= 0.0);
            assert!(layout.name_width <= 300.0);
        }
    }
}
