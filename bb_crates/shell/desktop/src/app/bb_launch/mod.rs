use crate::args::{Launch, Start};
use crate::editor::{Quit, Tabs};
use crate::theme::{install_appearance_observer, resolved_theme};
use crate::{ExplorerView, MoveDown, MoveLeft, MoveRight, MoveUp, Open, RestartServer};
use bb_explorer::Explorer;
use gpui::{
    px, size, App, AppContext, Bounds, FocusHandle, KeyBinding, Window, WindowAppearance,
    WindowBounds, WindowOptions,
};
use std::{path::PathBuf, time::Instant};

mod bb_window;
pub(crate) mod open;
pub(crate) mod recent;
pub(crate) use bb_window::{
    languages_found, menus, on_view as window_actions, title as window_title,
};
pub(crate) use open::langs::{self, Resolved, WindowLangs};
pub(crate) use open::{show_start_page, start_state};

pub(crate) fn launch() {
    // First, before any thread or child process (BB-BUG-009).
    bb_process::shell_env::adopt_login_path();
    // Then, still alone: end what an earlier backback left, and start the
    // guard that ends this one's servers however it ends (BB-PRC-009).
    adopt_children();
    let wanted: Launch = crate::args::parse_args();
    if let Launch::Workspace(start) = &wanted {
        langs::walk_early(&start.root, start.ticked);
        if let Err(error) = Explorer::open(&start.root) {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
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
        open::register(cx, wanted.no_write());
        crate::settings::install(cx);
        crate::settings::register_zoom(cx);
        cx.set_menus(menus(bb_lang::LangSet::EMPTY));
        // BB-FEAT-080: the pixel mascot in the Dock, with no macOS tile.
        if let Err(error) = bb_dock::set_icon(include_bytes!("../../../bb_assets/dock.png")) {
            eprintln!("bb: {error}");
        }
        match &wanted {
            Launch::Workspace(start) => {
                open_workspace(start, cx);
            }
            Launch::StartPage { no_write } => open::show_start_page(*no_write, cx),
        }
        cx.activate(true);
    });
}

/// The startup sweep and the guard; a line only when something was found.
fn adopt_children() {
    let Some(dir): Option<PathBuf> = recent::state_dir() else {
        return;
    };
    match bb_process::adopt(&dir.join("children")) {
        Ok(swept) if swept.ended.is_empty() && swept.refused.is_empty() => {}
        Ok(swept) => eprintln!(
            "bb_orphans ended={:?} refused={:?}",
            swept.ended, swept.refused
        ),
        Err(error) => eprintln!("bb: child records: {error}"),
    }
}

/// Open `wanted` in a new window and remember it as the newest workspace;
/// false when it cannot open.
pub(crate) fn open_workspace(wanted: &Start, cx: &mut App) -> bool {
    let start: Instant = Instant::now();
    let tree: Explorer = match Explorer::open(&wanted.root) {
        Ok(tree) => tree,
        Err(error) => {
            eprintln!("{error}");
            return false;
        }
    };
    recent::remember(&wanted.root);
    let bounds: Bounds<gpui::Pixels> = Bounds::centered(None, size(px(1000.0), px(700.0)), cx);
    let result: gpui::Result<gpui::WindowHandle<ExplorerView>> = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(bb_window::titlebar(&wanted.root)),
            ..Default::default()
        },
        |window: &mut Window, cx: &mut App| build_window(tree, wanted, start, window, cx),
    );
    if let Err(error) = &result {
        eprintln!("{error}");
    }
    result.is_ok()
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
    // A titled window is listed in the Window menu (BB-BUG-048).
    window.set_window_title(&bb_window::title(&wanted.root));
    bb_window::install_close_guard(window, &view, cx);
    install_appearance_observer(window, &view, cx);
    view.update(
        cx,
        |state: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            let later: Option<langs::Pending> = match langs::resolve(&wanted.root, wanted.ticked) {
                Resolved::Now(found) => {
                    state.langs = WindowLangs::new(found);
                    None
                }
                Resolved::Later(pending) => {
                    state.langs = WindowLangs::detecting();
                    Some(pending)
                }
            };
            bb_window::follow_languages(state, window, cx);
            match later {
                None => bb_window::start_tools(state, window, cx),
                // BB-PRF-009: the window shows now; the rest follows the walk.
                Some(pending) => langs::when_found(pending, window, cx),
            }
            crate::editor::watch_disk(window, cx);
            match &wanted.file {
                Some(file) => state.open_file(file, window, cx),
                // No file to wait for: the readback steps start now (BB-TG-016).
                None => crate::latency::drive(state, window, cx),
            }
        },
    );
    window.on_next_frame(move |_: &mut Window, _: &mut App| {
        eprintln!("first_frame_ms={}", start.elapsed().as_millis());
    });
    view
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
        lines: crate::editor::Lines::default(),
        theme: resolved_theme(None, Some(appearance)),
        theme_override: None,
        appearance_subscription: None,
        status: None,
        explorer_scroll: gpui::UniformListScrollHandle::new(),
        after_close: crate::editor::AfterClose::Window,
        root: root.clone(),
        intel: crate::intel::model::IntelModel::new(crate::intel::model::LoadStatus::NoSolution),
        link: None,
        rust: crate::intel::model::RustIntel::off(),
        python: crate::intel::model::PythonIntel::off(),
        tests: crate::tests_panel::TestsState::new(),
        debug: crate::run::bb_debug::DebugState::load(&root),
        quick_open: None,
        quick_focus: cx.focus_handle(),
        file_menu: crate::files::Menus::default(),
        terminal: crate::terminal::Terminal::new(),
        title_drag: crate::editor::TitleDrag::default(),
        langs: WindowLangs::new(bb_lang::Langs::csharp_only()),
    }
}

/// The app's key bindings; window tests install them to prove a key still
/// reaches the editor (BB-UIX-010).
pub(crate) fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", MoveUp, Some("Explorer")),
        KeyBinding::new("down", MoveDown, Some("Explorer")),
        KeyBinding::new("left", MoveLeft, Some("Explorer")),
        KeyBinding::new("right", MoveRight, Some("Explorer")),
        KeyBinding::new("enter", Open, Some("Explorer")),
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("cmd-shift-r", RestartServer, None),
        KeyBinding::new("cmd-shift-b", crate::run::build::BuildSolution, None),
        KeyBinding::new("f9", crate::run::bb_debug::ToggleBreakpoint, None),
        KeyBinding::new("f5", crate::run::bb_debug::StartOrContinue, None),
        KeyBinding::new("shift-f5", crate::run::bb_debug::StopDebugging, None),
        KeyBinding::new("f10", crate::run::bb_debug::StepOver, None),
        KeyBinding::new(
            "ctrl-shift-f10",
            crate::run::bb_debug::SetNextStatement,
            None,
        ),
        KeyBinding::new("f11", crate::run::bb_debug::StepInto, None),
        KeyBinding::new("shift-f11", crate::run::bb_debug::StepOut, None),
        KeyBinding::new("ctrl-`", crate::terminal::ToggleTerminal, None),
        KeyBinding::new("ctrl-shift-`", crate::terminal::SplitTerminal, None),
        KeyBinding::new("cmd-\\", crate::terminal::SplitTerminal, Some("Terminal")),
    ]);
    cx.bind_keys(bb_window::bindings());
    cx.bind_keys(open::bindings());
    cx.bind_keys(crate::editor::tab_bindings());
    cx.bind_keys(crate::settings::zoom_bindings());
}

#[cfg(test)]
mod tests;
