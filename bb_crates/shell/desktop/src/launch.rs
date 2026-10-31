use crate::editor::Tabs;
use crate::{ExplorerView, MoveDown, MoveLeft, MoveRight, MoveUp, Open};
use bb_explorer::Explorer;
use gpui::{
    px, size, App, AppContext, Bounds, FocusHandle, KeyBinding, Window, WindowBounds, WindowOptions,
};
use std::{path::PathBuf, time::Instant};

pub(crate) fn launch() {
    let start: Instant = Instant::now();
    let path: PathBuf = std::env::args_os()
        .nth(1)
        .map_or_else(|| PathBuf::from("."), PathBuf::from);
    let tree: Explorer = match Explorer::open(&path) {
        Ok(tree) => tree,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    gpui_platform::application().run(move |cx: &mut App| {
        bind_keys(cx);
        let bounds: Bounds<gpui::Pixels> = Bounds::centered(None, size(px(1000.0), px(700.0)), cx);
        let result: gpui::Result<gpui::WindowHandle<ExplorerView>> = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window: &mut Window, cx: &mut App| {
                let view: gpui::Entity<ExplorerView> =
                    cx.new(|cx: &mut gpui::Context<ExplorerView>| ExplorerView {
                        tree,
                        selected: 0,
                        focus: cx.focus_handle(),
                        editor_focus: cx.focus_handle(),
                        tabs: Tabs::new(),
                        keypresses: 0,
                        latency_before: None,
                        closing_window: false,
                        panel_width: 300.0,
                        resizing: false,
                    });
                let focus: FocusHandle = view.read(cx).focus.clone();
                window.focus(&focus, cx);
                install_close_guard(window, &view, cx);
                window.on_next_frame(move |_: &mut Window, _: &mut App| {
                    eprintln!("first_frame_ms={}", start.elapsed().as_millis());
                });
                view
            },
        );
        if let Err(error) = result {
            eprintln!("{error}");
            cx.quit();
        }
        cx.activate(true);
    });
}

fn install_close_guard(window: &Window, view: &gpui::Entity<ExplorerView>, cx: &App) {
    let close_view: gpui::Entity<ExplorerView> = view.clone();
    window.on_window_should_close(cx, move |_: &mut Window, cx: &mut App| {
        close_view.update(
            cx,
            |state: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
                state.handle_window_close(cx)
            },
        )
    });
}

fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", MoveUp, Some("Explorer")),
        KeyBinding::new("down", MoveDown, Some("Explorer")),
        KeyBinding::new("left", MoveLeft, Some("Explorer")),
        KeyBinding::new("right", MoveRight, Some("Explorer")),
        KeyBinding::new("enter", Open, Some("Explorer")),
    ]);
}
