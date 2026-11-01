//! BBEOD08: one font and size for every surface; BBEOD14: zoom keys.

fn sources(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for item in std::fs::read_dir(dir).unwrap() {
        let path: std::path::PathBuf = item.unwrap().path();
        if path.is_dir() {
            sources(&path, out);
        } else if path
            .extension()
            .is_some_and(|ext: &std::ffi::OsStr| ext == "rs")
        {
            out.push(path);
        }
    }
}

/// Only the window root sets the font, from the settings: a literal family
/// or size anywhere else makes that surface differ (BB-BUG-002).
#[test]
fn no_surface_sets_its_own_font_family_or_text_size() {
    let root: std::path::PathBuf = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    sources(&root, &mut files);
    let literal: [&str; 3] = [
        concat!("font_fam", "ily(\""),
        concat!(".text_", "size(px("),
        concat!("gpui::fo", "nt(\""),
    ];
    let offenders: Vec<String> = files
        .iter()
        .filter(|path: &&std::path::PathBuf| {
            let text: String = std::fs::read_to_string(path).unwrap();
            let body: &str = text.split("#[cfg(test)]").next().unwrap_or_default();
            literal.iter().any(|needle: &&str| body.contains(needle))
        })
        .map(|path: &std::path::PathBuf| path.display().to_string())
        .collect();
    assert!(files.len() > 20);
    assert_eq!(offenders, Vec::<String>::new());
}

fn bound(stroke: &'static str, action: &dyn gpui::Action) -> bool {
    let typed: gpui::Keystroke = gpui::Keystroke::parse(stroke).unwrap();
    super::zoom_bindings()
        .iter()
        .any(|binding: &gpui::KeyBinding| {
            binding.match_keystrokes(std::slice::from_ref(&typed)) == Some(false)
                && binding.action().partial_eq(action)
        })
}

#[test]
fn cmd_plus_minus_and_zero_zoom_the_whole_app() {
    assert!(bound("cmd-=", &super::keys::ZoomIn));
    assert!(bound("cmd--", &super::keys::ZoomOut));
    assert!(bound("cmd-0", &super::keys::ZoomReset));
}

#[test]
fn cmd_comma_opens_settings() {
    assert!(bound("cmd-,", &super::keys::OpenSettings));
}

fn click(vcx: &mut gpui::VisualTestContext, id: &'static str) {
    let at: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds(id).unwrap_or_else(|| panic!("no {id}"));
    vcx.simulate_click(at.center(), gpui::Modifiers::default());
}

fn fonts(vcx: &mut gpui::VisualTestContext) -> bb_ui::Typography {
    vcx.update(|_: &mut gpui::Window, cx: &mut gpui::App| super::typography(cx))
}

/// BBEOD08: the Settings window changes size, family and the tool window
/// side live; Reset brings back Menlo 13 on the left.
#[gpui::test]
fn the_settings_window_changes_size_family_and_side_live(cx: &mut gpui::TestAppContext) {
    let (_view, vcx): (
        gpui::Entity<super::view::SettingsWindow>,
        &mut gpui::VisualTestContext,
    ) = cx.add_window_view(
        |_: &mut gpui::Window, _: &mut gpui::Context<super::view::SettingsWindow>| {
            super::view::SettingsWindow::with_families(vec!["Menlo".into(), "Monaco".into()])
        },
    );
    vcx.run_until_parked();
    click(vcx, "size-up");
    assert_eq!(fonts(vcx).base, 14.0);
    click(vcx, "font-Monaco");
    assert_eq!(fonts(vcx).family, "Monaco");
    click(vcx, "side-right");
    assert!(fonts(vcx).panel_right);
    click(vcx, "reset");
    assert_eq!(fonts(vcx), bb_ui::Typography::default());
}
