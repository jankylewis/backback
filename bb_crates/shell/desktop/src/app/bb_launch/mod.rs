use crate::args::Start;
use crate::editor::{Quit, Tabs};
use crate::theme::{install_appearance_observer, resolved_theme, FollowAppearance, ToggleTheme};
use crate::{ExplorerView, MoveDown, MoveLeft, MoveRight, MoveUp, Open, RestartServer};
use bb_explorer::Explorer;
use gpui::{
    px, size, App, AppContext, Bounds, FocusHandle, KeyBinding, Menu, MenuItem, Window,
    WindowAppearance, WindowBounds, WindowOptions,
};
use std::{path::PathBuf, time::Instant};

mod bb_window;
pub(crate) use bb_window::on_view as window_actions;

pub(crate) fn launch() {
    let start: Instant = Instant::now();
    let wanted: Start = crate::args::parse_args();
    let tree: Explorer = match Explorer::open(&wanted.root) {
        Ok(tree) => tree,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    if let Err(error) = bb_process::exit_on_signal(true) {
        eprintln!("bb: cannot watch fatal signals: {error}");
    }
    gpui_platform::application().run(move |cx: &mut App| {
        // Quit ends every owned child group (Roslyn, test hosts, debugger);
        // gpui gives quit hooks 200 ms.
        cx.on_app_quit(|_: &mut App| {
            bb_process::shutdown();
            std::future::ready(())
        })
        .detach();
        bind_keys(cx);
        bb_window::register(cx);
        cx.set_menus(menus());
        let bounds: Bounds<gpui::Pixels> = Bounds::centered(None, size(px(1000.0), px(700.0)), cx);
        let result: gpui::Result<gpui::WindowHandle<ExplorerView>> = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window: &mut Window, cx: &mut App| build_window(tree, &wanted, start, window, cx),
        );
        if let Err(error) = result {
            eprintln!("{error}");
            cx.quit();
        }
        cx.activate(true);
    });
}

fn build_window(
    tree: Explorer,
    wanted: &Start,
    start: Instant,
    window: &mut Window,
    cx: &mut App,
) -> gpui::Entity<ExplorerView> {
    let workspace: (PathBuf, bool) = (wanted.root.clone(), wanted.no_write);
    let view: gpui::Entity<ExplorerView> = new_view(tree, workspace, window.appearance(), cx);
    let focus: FocusHandle = view.read(cx).focus.clone();
    window.focus(&focus, cx);
    install_close_guard(window, &view, cx);
    install_appearance_observer(window, &view, cx);
    view.update(
        cx,
        |state: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            crate::intel::glue::connect(state, window, cx);
            if let Some(file) = &wanted.file {
                state.open_file(file, window, cx);
            }
        },
    );
    window.on_next_frame(move |_: &mut Window, _: &mut App| {
        eprintln!("first_frame_ms={}", start.elapsed().as_millis());
    });
    view
}

fn menus() -> Vec<Menu> {
    vec![
        Menu::new("Backback").items(
            bb_window::app_items()
                .into_iter()
                .chain([MenuItem::action("Quit Backback", Quit)]),
        ),
        Menu::new("View").items([
            MenuItem::action("Toggle Light/Dark Theme", ToggleTheme),
            MenuItem::action("Follow macOS Appearance", FollowAppearance),
        ]),
        Menu::new("C#").items([MenuItem::action("Restart Server", RestartServer)]),
        bb_window::window_menu(),
    ]
}

fn new_view(
    tree: Explorer,
    workspace: (PathBuf, bool),
    appearance: WindowAppearance,
    cx: &mut App,
) -> gpui::Entity<ExplorerView> {
    cx.new(|cx: &mut gpui::Context<ExplorerView>| fresh_view(tree, workspace, appearance, cx))
}

/// The window's state for `tree`; `no_write` locks every save.
pub(crate) fn fresh_view(
    tree: Explorer,
    (root, no_write): (PathBuf, bool),
    appearance: WindowAppearance,
    cx: &mut gpui::Context<ExplorerView>,
) -> ExplorerView {
    let mut tabs: Tabs = Tabs::new();
    if no_write {
        tabs.set_no_write();
    }
    ExplorerView {
        tree,
        selected: 0,
        focus: cx.focus_handle(),
        editor_focus: cx.focus_handle(),
        tabs,
        keypresses: 0,
        latency_before: None,
        frame_collector: None,
        edit_ns: 0,
        reparse_ns: 0,
        closing_window: false,
        panel_width: 300.0,
        resizing: false,
        theme: resolved_theme(None, Some(appearance)),
        theme_override: None,
        appearance_subscription: None,
        status: None,
        explorer_scroll: gpui::UniformListScrollHandle::new(),
        quit_after_close: false,
        root: root.clone(),
        intel: crate::intel::model::IntelModel::new(crate::intel::model::LoadStatus::NoSolution),
        link: None,
        editor_scroll: gpui::UniformListScrollHandle::new(),
        tests: crate::tests_panel::TestsState::new(),
        pending_test_jump: None,
    }
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
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("cmd-shift-r", RestartServer, None),
    ]);
    cx.bind_keys(bb_window::bindings());
}

#[cfg(test)]
mod tests;
