//! The standard macOS window keys (BB-BUG-001): Cmd+M minimizes, Cmd+H hides
//! the app, Cmd+Alt+H hides the others; a Window menu with Minimize and Zoom.

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
