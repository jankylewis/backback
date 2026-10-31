use super::*;
use gpui::{Menu, MenuItem};

#[test]
fn a_file_argument_opens_its_folder_and_the_file() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let (folder, file): (PathBuf, PathBuf) = (dir.path().to_path_buf(), dir.path().join("A.cs"));
    std::fs::write(&file, "class A {}").unwrap();
    assert_eq!(split_argument(file.clone()), (folder.clone(), Some(file)));
    assert_eq!(split_argument(folder.clone()), (folder, None));
}

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
        Some(MenuItem::Action { name, .. }) if name.as_ref() == "Quit Backback"
    ));
    assert_eq!(configured[1].name.as_ref(), "View");
    assert_eq!(configured[1].items.len(), 3);
}
