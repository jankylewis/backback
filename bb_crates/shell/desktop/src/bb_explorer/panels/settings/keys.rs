//! Keys and menu items for the settings: zoom (BBEOD14) - Cmd+= bigger,
//! Cmd+- smaller, Cmd+0 back to the Settings size, as in Ghostty and VS
//! Code - the tool window side, and Settings... (Cmd+,).

use bb_ui::Typography;
use gpui::{App, KeyBinding, MenuItem};

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct ZoomIn;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct ZoomOut;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct ZoomReset;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct ToggleToolWindowSide;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct OpenSettings;

pub(crate) fn register(cx: &mut App) {
    cx.on_action(|_: &ZoomIn, cx: &mut App| super::bb_store::update(cx, Typography::zoom_in));
    cx.on_action(|_: &ZoomOut, cx: &mut App| super::bb_store::update(cx, Typography::zoom_out));
    cx.on_action(|_: &ZoomReset, cx: &mut App| super::bb_store::update(cx, Typography::zoom_reset));
    cx.on_action(|_: &ToggleToolWindowSide, cx: &mut App| {
        super::bb_store::update(cx, |t: &mut Typography| t.panel_right = !t.panel_right);
    });
    cx.on_action(|_: &OpenSettings, cx: &mut App| super::view::open(cx));
}

pub(crate) fn bindings() -> [KeyBinding; 5] {
    [
        KeyBinding::new("cmd-=", ZoomIn, None),
        KeyBinding::new("cmd-+", ZoomIn, None),
        KeyBinding::new("cmd--", ZoomOut, None),
        KeyBinding::new("cmd-0", ZoomReset, None),
        KeyBinding::new("cmd-,", OpenSettings, None),
    ]
}

/// View menu: zoom and the tool window side.
pub(crate) fn menu_items() -> [MenuItem; 5] {
    [
        MenuItem::action("Zoom In", ZoomIn),
        MenuItem::action("Zoom Out", ZoomOut),
        MenuItem::action("Actual Size", ZoomReset),
        MenuItem::separator(),
        MenuItem::action("Move Tool Window to Other Side", ToggleToolWindowSide),
    ]
}

/// App menu: Settings...
pub(crate) fn settings_item() -> MenuItem {
    MenuItem::action("Settings...", OpenSettings)
}
