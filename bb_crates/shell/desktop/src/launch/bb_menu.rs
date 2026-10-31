use super::{FollowAppearance, ToggleHarness, ToggleTheme};
use crate::editor::Quit;
use gpui::{Menu, MenuItem};

pub(super) fn menus() -> Vec<Menu> {
    vec![
        Menu::new("Backback").items([MenuItem::action("Quit Backback", Quit)]),
        Menu::new("View").items([
            MenuItem::action("Toggle Light/Dark Theme", ToggleTheme),
            MenuItem::action("Follow macOS Appearance", FollowAppearance),
            MenuItem::action("Toggle Harness", ToggleHarness),
        ]),
    ]
}
