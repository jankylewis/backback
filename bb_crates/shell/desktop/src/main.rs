//! macOS desktop shell for the backback Explorer.
#[path = "app/args.rs"]
mod args;
#[path = "bb_explorer/bb_row_view.rs"]
mod bb_row_view;
mod editor;
#[path = "bb_explorer/icons.rs"]
mod icons;
mod intel;
#[path = "app/latency.rs"]
mod latency;
#[path = "app/bb_launch.rs"]
mod launch;
#[path = "bb_explorer/navigation.rs"]
mod navigation;
#[path = "bb_explorer/tests_panel/panel.rs"]
mod panel;
#[path = "app/script.rs"]
mod script;
#[path = "bb_explorer/tests_panel/mod.rs"]
mod tests_panel;
#[path = "app/theme.rs"]
mod theme;
use theme::color;
#[path = "bb_explorer/tooltip.rs"]
mod tooltip;
use bb_explorer::{Entry, Explorer};
use bb_ui::bb_theme::{Theme, ThemeKind};
use editor::{Tab, TabError, Tabs};
use gpui::{
    div, prelude::*, AsyncWindowContext, Context, FocusHandle, IntoElement, MouseButton, Render,
    ScrollStrategy, UniformListScrollHandle, WeakEntity, Window,
};
use intel::glue::RestartServer;
use navigation::{MoveDown, MoveLeft, MoveRight, MoveUp, Open};
use std::path::PathBuf;
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
    /// The folder the Explorer shows; files outside it open read-only.
    root: PathBuf,
    intel: intel::model::IntelModel,
    link: Option<intel::bb_worker::Link>,
    editor_scroll: UniformListScrollHandle,
    tests: tests_panel::TestsState,
    pending_test_jump: Option<(PathBuf, usize)>,
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
                intel::glue::opened(self, file);
                tests_panel::navigation::apply(self, cx);
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
        intel::glue::flush(self);
        intel::view::note_frame(&mut self.intel);
        let status: String = self.intel.status.label();
        let panel: Option<gpui::AnyElement> = intel::view::panels::bottom(self, cx);
        div()
            .size_full()
            .flex()
            .bg(color(self.theme.bg))
            .on_action(cx.listener(ExplorerView::toggle_theme))
            .on_action(cx.listener(ExplorerView::follow_appearance))
            .on_mouse_move(cx.listener(Self::move_resize))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::finish_resize))
            .on_action(cx.listener(ExplorerView::quit))
            .on_action(cx.listener(ExplorerView::restart_server))
            .child(match self.tests.side {
                tests_panel::Side::Files => panel::explorer_panel(self, cx).into_any_element(),
                tests_panel::Side::Tests => tests_panel::panel(self, cx).into_any_element(),
            })
            .child(panel::resize_edge(self.theme, cx))
            .child(
                div()
                    .flex_1()
                    .min_w(gpui::px(0.0))
                    .overflow_hidden()
                    .h_full()
                    .flex()
                    .flex_col()
                    .child(editor::tab_bar(&self.tabs, self.theme, cx))
                    .child(editor::editor_body(self, cx))
                    .children(panel)
                    .child(editor::status_bar(
                        self.status.as_deref(),
                        &status,
                        self.intel.status == intel::model::LoadStatus::Stopped,
                        self.theme,
                        cx,
                    )),
            )
    }
}

fn main() {
    launch::launch();
}
