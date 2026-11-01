use super::*;
use bb_ui::bb_theme::ThemeKind;

#[test]
fn absent_preference_is_dark_and_system_light_is_live_source() {
    assert_eq!(resolved_theme(None, None).kind, ThemeKind::Dark);
    assert_eq!(
        resolved_theme(None, Some(WindowAppearance::Light)).kind,
        ThemeKind::Light
    );
    assert_eq!(
        resolved_theme(None, Some(WindowAppearance::Dark)).kind,
        ThemeKind::Dark
    );
    assert_eq!(
        resolved_theme(Some(ThemeKind::Dark), Some(WindowAppearance::Light)).kind,
        ThemeKind::Dark
    );
}

#[test]
fn view_is_a_separate_mac_menu_after_application_menu() {
    let configured: Vec<Menu> = menus();
    assert_eq!(configured[0].name.as_ref(), "Backback");
    assert!(matches!(
        configured[0].items.first(),
        Some(MenuItem::Action { name, .. }) if name.as_ref() == "Hide Backback"
    ));
    assert_eq!(configured[1].name.as_ref(), "View");
    assert_eq!(configured[1].items.len(), 2);
}

/// Headmate ruling 2026-11-01: no menu opens an MCP or Harness panel.
#[test]
fn no_menu_item_opens_an_mcp_panel() {
    for menu in menus() {
        for item in &menu.items {
            if let MenuItem::Action { name, .. } = item {
                let lower: String = name.to_lowercase();
                assert!(
                    !lower.contains("mcp") && !lower.contains("harness"),
                    "{name}"
                );
            }
        }
    }
}

fn names(menu: &Menu) -> Vec<String> {
    menu.items
        .iter()
        .filter_map(|item: &MenuItem| match item {
            MenuItem::Action { name, .. } => Some(name.to_string()),
            MenuItem::Separator | MenuItem::Submenu(_) | MenuItem::SystemMenu(_) => None,
        })
        .collect()
}

/// BB-BUG-001: the standard macOS window keys, as every Mac app has them.
fn bound(stroke: &'static str, action: &dyn gpui::Action) -> bool {
    let typed: gpui::Keystroke = gpui::Keystroke::parse(stroke).unwrap();
    bb_window::bindings().iter().any(|binding: &KeyBinding| {
        binding.match_keystrokes(std::slice::from_ref(&typed)) == Some(false)
            && binding.action().partial_eq(action)
    })
}

#[test]
fn cmd_m_minimizes_cmd_h_hides_and_cmd_alt_h_hides_others() {
    assert!(bound("cmd-m", &bb_window::Minimize));
    assert!(bound("cmd-h", &bb_window::Hide));
    assert!(bound("cmd-alt-h", &bb_window::HideOthers));
}

#[test]
fn the_app_menu_can_hide_and_a_window_menu_minimizes_and_zooms() {
    let configured: Vec<Menu> = menus();
    assert_eq!(
        names(&configured[0]),
        ["Hide Backback", "Hide Others", "Show All", "Quit Backback"]
    );
    let window: &Menu = configured
        .iter()
        .find(|menu: &&Menu| menu.name.as_ref() == "Window")
        .expect("a Window menu");
    assert_eq!(names(window), ["Minimize", "Zoom"]);
}
