//! The standard macOS window keys (BB-BUG-001): Cmd+M minimizes, Cmd+H hides
//! the app, Cmd+Alt+H hides the others; a Window menu with Minimize and Zoom.
//! The menu bar follows the active window's languages: Debug and C# only
//! while C# is active there (BBEOD33).

use gpui::{App, InteractiveElement, KeyBinding, Menu, MenuItem, Window};

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct Minimize;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct Zoom;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct Hide;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct HideOthers;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct ShowAll;

/// App-wide handlers, so the keys work whichever view has focus.
pub(crate) fn register(cx: &mut App) {
    cx.on_action(|_: &Hide, cx: &mut App| cx.hide());
    cx.on_action(|_: &HideOthers, cx: &mut App| cx.hide_other_apps());
    cx.on_action(|_: &ShowAll, cx: &mut App| cx.unhide_other_apps());
}

/// Minimize and Zoom act on the window the key or menu reached, never on
/// `cx.active_window()` (None while another app is active).
pub(crate) fn on_view<E: InteractiveElement>(element: E) -> E {
    element
        .on_action(|_: &Minimize, window: &mut Window, _: &mut App| window.minimize_window())
        .on_action(|_: &Zoom, window: &mut Window, _: &mut App| window.zoom_window())
}

pub(crate) fn bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("cmd-m", Minimize, None),
        KeyBinding::new("cmd-h", Hide, None),
        KeyBinding::new("cmd-alt-h", HideOthers, None),
    ]
}

/// The app menu items before Quit.
pub(crate) fn app_items() -> [MenuItem; 6] {
    [
        MenuItem::action("Hide Backback", Hide),
        MenuItem::action("Hide Others", HideOthers),
        MenuItem::action("Show All", ShowAll),
        MenuItem::separator(),
        crate::settings::settings_item(),
        MenuItem::separator(),
    ]
}

pub(crate) fn window_menu() -> Menu {
    Menu::new("Window").items([
        MenuItem::action("Minimize", Minimize),
        MenuItem::action("Zoom", Zoom),
    ])
}

/// The system bar is hidden: backback draws its own, with Back / Forward
/// (BB-FEAT-090); the window buttons sit inside it.
pub(crate) fn titlebar(root: &std::path::Path) -> gpui::TitlebarOptions {
    gpui::TitlebarOptions {
        title: Some(title(root).into()),
        appears_transparent: true,
        traffic_light_position: Some(gpui::point(gpui::px(12.0), gpui::px(10.0))),
    }
}

/// A workspace window's title, the folder's name as VS Code shows it:
/// macOS lists only titled windows in the Window menu (BB-BUG-048).
pub(crate) fn title(root: &std::path::Path) -> String {
    root.file_name().map_or_else(
        || root.display().to_string(),
        |name: &std::ffi::OsStr| name.to_string_lossy().into_owned(),
    )
}

/// The window's close asks the view first (unsaved tabs ask).
pub(crate) fn install_close_guard(
    window: &gpui::Window,
    view: &gpui::Entity<crate::ExplorerView>,
    cx: &gpui::App,
) {
    let close_view: gpui::Entity<crate::ExplorerView> = view.clone();
    window.on_window_should_close(cx, move |_: &mut gpui::Window, cx: &mut gpui::App| {
        close_view.update(
            cx,
            |state: &mut crate::ExplorerView, cx: &mut gpui::Context<crate::ExplorerView>| {
                state.handle_window_close(cx)
            },
        )
    });
}

/// The menu bar; `csharp`: C# is active in the front window, so the C#-only
/// menus (Debug, C#) show (BBEOD33; they showed in every window before).
pub(crate) fn menus(csharp: bool) -> Vec<Menu> {
    use crate::theme::{FollowAppearance, ToggleTheme};
    let view: Menu = Menu::new("View").items(
        [
            MenuItem::action("Toggle Light/Dark Theme", ToggleTheme),
            MenuItem::action("Follow macOS Appearance", FollowAppearance),
            MenuItem::separator(),
            // Ctrl+` shows it, a second time hides it (BB-BUG-047).
            MenuItem::action("Terminal", crate::terminal::ToggleTerminal),
            MenuItem::action("Split Terminal", crate::terminal::SplitTerminal),
            MenuItem::separator(),
        ]
        .into_iter()
        .chain(crate::settings::zoom_menu_items()),
    );
    let build: Menu = Menu::new("Build").items([
        MenuItem::action("Build Solution", crate::run::build::BuildSolution),
        MenuItem::action("Cargo Build", crate::run::build::CargoBuild),
        MenuItem::action("Cargo Check", crate::run::build::CargoCheck),
        MenuItem::action("Python Check", crate::run::build::PythonCheck),
    ]);
    let quit: MenuItem = MenuItem::action("Quit Backback", crate::editor::Quit);
    let mut bar: Vec<Menu> = vec![
        Menu::new("Backback").items(app_items().into_iter().chain([quit])),
        super::open::file_menu(),
        view,
        build,
    ];
    if csharp {
        bar.push(crate::run::bb_debug::debug_menu());
        bar.push(Menu::new("C#").items([MenuItem::action("Restart Server", crate::RestartServer)]));
    }
    bar.push(window_menu());
    bar
}

/// Show the menu bar of `view`'s languages now (its window opens in front)
/// and each time its window comes to the front again.
pub(crate) fn follow_languages(
    view: &mut crate::ExplorerView,
    window: &mut Window,
    cx: &mut gpui::Context<crate::ExplorerView>,
) {
    cx.set_menus(menus(view.langs.langs.debug_visible()));
    cx.observe_window_activation(
        window,
        |view: &mut crate::ExplorerView,
         window: &mut Window,
         cx: &mut gpui::Context<crate::ExplorerView>| {
            if window.is_window_active() {
                cx.set_menus(menus(view.langs.langs.debug_visible()));
            }
        },
    )
    .detach();
}

/// Start the tools the window's languages allow.
pub(super) fn start_tools(
    state: &mut crate::ExplorerView,
    window: &mut Window,
    cx: &mut gpui::Context<crate::ExplorerView>,
) {
    crate::intel::glue::connect(state, window, cx);
    crate::intel::glue::connect_rust(state, window, cx);
    crate::intel::glue::connect_python(state, window, cx);
    crate::rows::cargo::load(state, window, cx);
}

/// BB-PRF-009: the launch's walk ended after the window showed. Its menus,
/// tools and tab colours follow the languages; every open tab is sent to
/// the servers that just started.
pub(crate) fn languages_found(
    state: &mut crate::ExplorerView,
    window: &mut Window,
    cx: &mut gpui::Context<crate::ExplorerView>,
) {
    if window.is_window_active() {
        cx.set_menus(menus(state.langs.langs.debug_visible()));
    }
    start_tools(state, window, cx);
    crate::editor::recolour(state);
    crate::intel::glue::resync_open(state);
    cx.notify();
}
