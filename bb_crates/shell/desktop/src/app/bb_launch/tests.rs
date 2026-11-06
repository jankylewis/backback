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
fn menus_go_app_then_file_then_view_as_on_every_mac() {
    let configured: Vec<Menu> = menus();
    assert_eq!(configured[0].name.as_ref(), "Backback");
    assert!(matches!(
        configured[0].items.first(),
        Some(MenuItem::Action { name, .. }) if name.as_ref() == "Hide Backback"
    ));
    assert_eq!(configured[1].name.as_ref(), "File");
    assert_eq!(configured[2].name.as_ref(), "View");
    let view: Vec<String> = names(&configured[2]);
    assert_eq!(view.len(), 7, "{view:?}");
    // BB-BUG-047: View -> Terminal.
    assert!(
        view.iter().any(|name: &String| name == "Terminal"),
        "{view:?}"
    );
    assert!(
        view.iter()
            .any(|name: &String| name == "Move Tool Window to Other Side"),
        "{view:?}"
    );
    assert!(
        view.iter().any(|name: &String| name == "Zoom In"),
        "{view:?}"
    );
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
        [
            "Hide Backback",
            "Hide Others",
            "Show All",
            "Settings...",
            "Quit Backback"
        ]
    );
    let window: &Menu = configured
        .iter()
        .find(|menu: &&Menu| menu.name.as_ref() == "Window")
        .expect("a Window menu");
    assert_eq!(names(window), ["Minimize", "Zoom"]);
}

/// BBEOD13: File > Open... and Cmd+O, as every Mac app has them.
#[test]
fn cmd_o_and_the_file_menu_open_a_workspace() {
    let typed: gpui::Keystroke = gpui::Keystroke::parse("cmd-o").unwrap();
    let binding: &KeyBinding = &open::bindings()[0];
    assert_eq!(
        binding.match_keystrokes(std::slice::from_ref(&typed)),
        Some(false)
    );
    assert!(binding.action().partial_eq(&open::OpenWorkspace));
    let go: &KeyBinding = &open::bindings()[1];
    let typed_go: gpui::Keystroke = gpui::Keystroke::parse("cmd-shift-o").unwrap();
    assert_eq!(
        go.match_keystrokes(std::slice::from_ref(&typed_go)),
        Some(false)
    );
    assert!(go.action().partial_eq(&crate::quick_open::GoToFile));
    let file: Menu = menus()
        .into_iter()
        .find(|menu: &Menu| menu.name.as_ref() == "File")
        .expect("a File menu");
    assert_eq!(
        names(&file),
        [
            "New Window",
            "Open...",
            "Close Folder",
            "Start Page...",
            "Go to File...",
            "Go to Symbol...",
            "Find in Files...",
            "Back",
            "Forward"
        ]
    );
    let back: &KeyBinding = &open::bindings()[2];
    let typed_back: gpui::Keystroke = gpui::Keystroke::parse("cmd-[").unwrap();
    assert_eq!(
        back.match_keystrokes(std::slice::from_ref(&typed_back)),
        Some(false)
    );
    assert!(back.action().partial_eq(&crate::editor::GoBack));
    let find: &KeyBinding = &open::bindings()[5];
    let typed_find: gpui::Keystroke = gpui::Keystroke::parse("cmd-shift-f").unwrap();
    assert_eq!(
        find.match_keystrokes(std::slice::from_ref(&typed_find)),
        Some(false)
    );
    assert!(find.action().partial_eq(&crate::quick_open::FindInFiles));
}

/// Open `argument` as a workspace window; every window after it.
fn open_one(
    argument: std::path::PathBuf,
    cx: &mut gpui::TestAppContext,
) -> Vec<gpui::AnyWindowHandle> {
    cx.update(bind_keys);
    let start: Start = crate::args::start_for(argument, false);
    assert!(cx.update(|cx: &mut App| super::open_workspace(&start, cx)));
    cx.run_until_parked();
    cx.windows()
}

/// (workspace windows, start pages) now open.
fn counts(cx: &mut gpui::TestAppContext) -> (usize, usize) {
    let windows: Vec<gpui::AnyWindowHandle> = cx.windows();
    let views: usize = windows
        .iter()
        .filter(|w: &&gpui::AnyWindowHandle| w.downcast::<ExplorerView>().is_some())
        .count();
    (views, windows.len().saturating_sub(views))
}

/// File > Close Folder (Cmd+K F), as VS Code: the workspace closes and the
/// start page shows.
#[gpui::test]
fn close_folder_closes_the_workspace_and_shows_the_start_page(cx: &mut gpui::TestAppContext) {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let windows: Vec<gpui::AnyWindowHandle> = open_one(dir.path().to_path_buf(), cx);
    let window: gpui::AnyWindowHandle = *windows.first().unwrap();
    assert_eq!(counts(cx), (1, 0));
    let mut wcx: gpui::VisualTestContext = gpui::VisualTestContext::from_window(window, cx);
    wcx.simulate_keystrokes("cmd-k f");
    wcx.run_until_parked();
    assert_eq!(
        counts(cx),
        (0, 1),
        "the workspace window closed and the start page shows"
    );
}

/// Close Folder with an unsaved tab asks first: the window stays, no start
/// page yet.
#[gpui::test]
fn close_folder_with_an_unsaved_tab_asks_first(cx: &mut gpui::TestAppContext) {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let file: std::path::PathBuf = dir.path().join("A.cs");
    std::fs::write(&file, "class A {}\n").unwrap();
    let windows: Vec<gpui::AnyWindowHandle> = open_one(file, cx);
    let window: gpui::AnyWindowHandle = *windows.first().unwrap();
    let mut wcx: gpui::VisualTestContext = gpui::VisualTestContext::from_window(window, cx);
    wcx.simulate_keystrokes("x cmd-k f");
    wcx.run_until_parked();
    assert_eq!(counts(cx), (1, 0), "the save prompt holds the window");
    let asking: bool = window
        .downcast::<ExplorerView>()
        .unwrap()
        .read_with(cx, |v: &ExplorerView, _: &App| {
            v.closing_window && v.after_close == crate::editor::AfterClose::StartPage
        })
        .unwrap();
    assert!(asking, "the save prompt is up, for Close Folder");
}
