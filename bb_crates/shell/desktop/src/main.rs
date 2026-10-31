//! macOS desktop shell for the backback Explorer.

#[path = "bb_explorer/bb_row_view.rs"]
mod bb_row_view;
mod editor;
#[path = "bb_explorer/icons.rs"]
mod icons;
mod latency;
mod launch;
#[path = "bb_explorer/navigation.rs"]
mod navigation;
#[path = "bb_explorer/tooltip.rs"]
mod tooltip;

use std::path::PathBuf;

use bb_explorer::{Entry, Explorer};
use bb_row_view::{row, RowState};
use bb_ui::{
    bb_theme::{Theme, ThemeKind},
    Rgb,
};
use editor::Tabs;
use gpui::{
    div, prelude::*, px, rgb, Context, FocusHandle, InteractiveElement, IntoElement, MouseButton,
    Render, StatefulInteractiveElement, Window,
};

/// Select the preceding Explorer row.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
struct MoveUp;
/// Select the next Explorer row.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
struct MoveDown;
/// Collapse the selected folder or select its parent.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
struct MoveLeft;
/// Expand the selected folder.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
struct MoveRight;
/// Open the selected entry.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
struct Open;

struct ExplorerView {
    tree: Explorer,
    selected: usize,
    focus: FocusHandle,
    editor_focus: FocusHandle,
    tabs: Tabs,
    keypresses: usize,
    latency_before: Option<gpui::profiler::InputLatencySnapshot>,
    frame_collector: Option<gpui::profiler::FrameTimingCollector>,
    edit_ns: u64,
    reparse_ns: u64,
    closing_window: bool,
    panel_width: f32,
    resizing: bool,
    theme: Theme,
    theme_override: Option<ThemeKind>,
    appearance_subscription: Option<gpui::Subscription>,
}

impl ExplorerView {
    fn editor_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        editor::handle_key(self, event, window, cx);
    }

    fn move_resize(
        &mut self,
        event: &gpui::MouseMoveEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.resizing && event.dragging() {
            self.panel_width = f32::from(event.position.x).clamp(180.0, 800.0);
            cx.notify();
        }
    }

    fn finish_resize(&mut self, _: &gpui::MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.resizing = false;
    }

    fn start_resize(&mut self, _: &gpui::MouseDownEvent, _: &mut Window, _: &mut Context<Self>) {
        self.resizing = true;
    }

    fn move_up(&mut self, _: &MoveUp, _: &mut Window, cx: &mut Context<Self>) {
        self.selected = self.selected.saturating_sub(1);
        cx.notify();
    }

    fn move_down(&mut self, _: &MoveDown, _: &mut Window, cx: &mut Context<Self>) {
        self.selected = self
            .selected
            .saturating_add(1)
            .min(self.tree.visible().len().saturating_sub(1));
        cx.notify();
    }

    fn move_left(&mut self, _: &MoveLeft, _: &mut Window, cx: &mut Context<Self>) {
        match navigation::left(&mut self.tree, self.selected) {
            Ok(selected) => self.selected = selected,
            Err(error) => eprintln!("{error}"),
        }
        cx.notify();
    }

    fn move_right(&mut self, _: &MoveRight, _: &mut Window, cx: &mut Context<Self>) {
        match navigation::right(&mut self.tree, self.selected) {
            Ok(selected) => self.selected = selected,
            Err(error) => eprintln!("{error}"),
        }
        cx.notify();
    }

    fn open(&mut self, _: &Open, window: &mut Window, cx: &mut Context<Self>) {
        self.open_selected(window, cx);
    }

    fn open_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let row: Option<(PathBuf, bool)> = self
            .tree
            .visible()
            .get(self.selected)
            .map(|(_, entry): &(usize, &Entry)| (entry.path().to_path_buf(), entry.expandable()));
        if let Some((path, folder)) = row {
            if folder {
                if let Err(error) = self.tree.toggle(&path) {
                    eprintln!("{error}");
                }
            } else {
                self.open_file(&path, window, cx);
            }
        }
        cx.notify();
    }

    fn open_file(&mut self, path: &std::path::Path, window: &mut Window, cx: &mut Context<Self>) {
        match self.tabs.open(path) {
            Ok(()) => window.focus(&self.editor_focus, cx),
            Err(error) => eprintln!("{error}"),
        }
        cx.notify();
    }
}

impl Render for ExplorerView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .bg(color(self.theme.bg))
            .on_action(cx.listener(ExplorerView::toggle_theme))
            .on_action(cx.listener(ExplorerView::follow_appearance))
            .on_mouse_move(cx.listener(Self::move_resize))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::finish_resize))
            .child(explorer_panel(self, cx))
            .child(resize_edge(self.theme, cx))
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .child(editor::tab_bar(&self.tabs, self.theme, cx))
                    .child(editor::editor_body(
                        &self.tabs,
                        &self.editor_focus,
                        self.theme,
                        cx,
                    )),
            )
    }
}

fn explorer_panel(view: &ExplorerView, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    let rows: Vec<(usize, &Entry)> = view.tree.visible();
    div()
        .id("explorer-panel")
        .w(px(view.panel_width))
        .h_full()
        .flex()
        .flex_col()
        .bg(color(view.theme.surface))
        .overflow_y_scroll()
        .key_context("Explorer")
        .track_focus(&view.focus)
        .on_action(cx.listener(ExplorerView::move_up))
        .on_action(cx.listener(ExplorerView::move_down))
        .on_action(cx.listener(ExplorerView::move_left))
        .on_action(cx.listener(ExplorerView::move_right))
        .on_action(cx.listener(ExplorerView::open))
        .child(
            div()
                .px_3()
                .py_2()
                .text_color(color(view.theme.text))
                .child("EXPLORER"),
        )
        .children(rows.into_iter().enumerate().map(
            |(index, (depth, entry)): (usize, (usize, &Entry))| {
                row(
                    index,
                    depth,
                    entry,
                    RowState {
                        selected: index == view.selected,
                        panel_width: view.panel_width,
                        theme: view.theme,
                    },
                    cx,
                )
            },
        ))
}

fn resize_edge(theme: Theme, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    div()
        .id("explorer-resize-edge")
        .w(px(6.0))
        .h_full()
        .flex_shrink_0()
        .bg(color(theme.border))
        .on_mouse_down(MouseButton::Left, cx.listener(ExplorerView::start_resize))
}

fn color(value: Rgb) -> gpui::Rgba {
    let packed: u32 = (u32::from(value.r) << 16) | (u32::from(value.g) << 8) | u32::from(value.b);
    rgb(packed)
}

fn main() {
    launch::launch();
}
