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
#[path = "bb_explorer/panel.rs"]
mod panel;
#[path = "bb_explorer/tooltip.rs"]
mod tooltip;

use std::path::PathBuf;

use bb_explorer::{Entry, Explorer};
use bb_ui::{
    bb_theme::{Theme, ThemeKind},
    Rgb,
};
use editor::{Tab, TabError, Tabs};
use gpui::{
    div, prelude::*, rgb, AsyncWindowContext, Context, FocusHandle, IntoElement, MouseButton,
    Render, ScrollStrategy, UniformListScrollHandle, WeakEntity, Window,
};

use navigation::{MoveDown, MoveLeft, MoveRight, MoveUp, Open};

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
    status: Option<String>,
    explorer_scroll: UniformListScrollHandle,
    quit_after_close: bool,
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

    /// Show an error in the status bar (and on stderr for logs).
    pub(crate) fn report(&mut self, error: &dyn std::fmt::Display) {
        let text: String = error.to_string();
        eprintln!("{text}");
        self.status = Some(text);
    }

    fn select_row(&mut self, selected: usize, cx: &mut Context<Self>) {
        self.selected = selected;
        self.explorer_scroll
            .scroll_to_item(selected, ScrollStrategy::Nearest);
        cx.notify();
    }

    fn move_up(&mut self, _: &MoveUp, _: &mut Window, cx: &mut Context<Self>) {
        self.select_row(self.selected.saturating_sub(1), cx);
    }

    fn move_down(&mut self, _: &MoveDown, _: &mut Window, cx: &mut Context<Self>) {
        let last: usize = self.tree.visible().len().saturating_sub(1);
        self.select_row(self.selected.saturating_add(1).min(last), cx);
    }

    fn move_left(&mut self, _: &MoveLeft, _: &mut Window, cx: &mut Context<Self>) {
        match navigation::left(&mut self.tree, self.selected) {
            Ok(selected) => self.select_row(selected, cx),
            Err(error) => self.report(&error),
        }
        cx.notify();
    }

    fn move_right(&mut self, _: &MoveRight, _: &mut Window, cx: &mut Context<Self>) {
        match navigation::right(&mut self.tree, self.selected) {
            Ok(selected) => self.select_row(selected, cx),
            Err(error) => self.report(&error),
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
                    self.report(&error);
                }
            } else {
                self.open_file(&path, window, cx);
            }
        }
        cx.notify();
    }

    /// Open a file without blocking the UI: a placeholder tab shows while a
    /// background thread reads and parses it.
    fn open_file(&mut self, path: &std::path::Path, window: &mut Window, cx: &mut Context<Self>) {
        match self.tabs.begin_open(path) {
            Ok(Some(file)) => {
                let job: PathBuf = file.clone();
                let load: gpui::Task<Result<Tab, TabError>> = cx
                    .background_executor()
                    .spawn(async move { Tab::load(job) });
                cx.spawn_in(
                    window,
                    async move |view: WeakEntity<Self>, cx: &mut AsyncWindowContext| {
                        let loaded: Result<Tab, TabError> = load.await;
                        view.update_in(
                            cx,
                            |view: &mut Self, window: &mut Window, cx: &mut Context<Self>| {
                                view.finish_open(&file, loaded, window, cx);
                            },
                        )
                        .ok();
                    },
                )
                .detach();
            }
            Ok(None) => window.focus(&self.editor_focus, cx),
            Err(error) => self.report(&error),
        }
        cx.notify();
    }

    fn finish_open(
        &mut self,
        file: &std::path::Path,
        loaded: Result<Tab, TabError>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match self.tabs.finish_open(file, loaded) {
            Ok(()) => {
                window.focus(&self.editor_focus, cx);
                latency::drive(self, window, cx);
            }
            Err(error) => self.report(&error),
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
            .on_action(cx.listener(ExplorerView::quit))
            .child(panel::explorer_panel(self, cx))
            .child(panel::resize_edge(self.theme, cx))
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
                    ))
                    .child(editor::status_bar(self.status.as_deref(), self.theme)),
            )
    }
}

fn color(value: Rgb) -> gpui::Rgba {
    let packed: u32 = (u32::from(value.r) << 16) | (u32::from(value.g) << 8) | u32::from(value.b);
    rgb(packed)
}

fn main() {
    launch::launch();
}
