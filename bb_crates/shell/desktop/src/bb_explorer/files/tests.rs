//! BB-BUG-050: the Explorer's New / Rename / Delete, on disk and through
//! the window (a right-click, the menu, the name box, the Trash question).

use std::path::{Path, PathBuf};

use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext};

use super::bb_ops::{new_file, new_folder, rename, trash_with, FileOpError};
use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
use crate::ExplorerView;

#[test]
fn a_new_file_or_folder_never_overwrites_and_names_are_checked() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let made: PathBuf = new_file(dir.path(), "  C.cs ").unwrap();
    assert_eq!(made, dir.path().join("C.cs"), "trimmed");
    assert!(matches!(
        new_file(dir.path(), "C.cs"),
        Err(FileOpError::Exists(_))
    ));
    for bad in ["", " ", ".", "..", "a/b"] {
        assert!(
            matches!(new_file(dir.path(), bad), Err(FileOpError::BadName(_))),
            "{bad:?}"
        );
    }
    assert!(new_folder(dir.path(), "Sub").unwrap().is_dir());
    assert!(matches!(
        new_folder(dir.path(), "Sub"),
        Err(FileOpError::Exists(_))
    ));
}

#[test]
fn a_rename_keeps_the_folder_and_never_overwrites() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let a: PathBuf = new_file(dir.path(), "a.cs").unwrap();
    let b: PathBuf = new_file(dir.path(), "b.cs").unwrap();
    assert!(matches!(rename(&a, "b.cs"), Err(FileOpError::Exists(_))));
    let upper: PathBuf = rename(&a, "A.cs").unwrap();
    assert!(upper.exists(), "a case-only rename works");
    let moved: PathBuf = rename(&b, "c.cs").unwrap();
    assert_eq!(moved, dir.path().join("c.cs"));
    assert!(!b.exists());
}

#[test]
fn the_trash_runs_its_program_and_a_refusal_is_an_error() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let gone: PathBuf = new_file(dir.path(), "x.txt").unwrap();
    trash_with(Path::new("/bin/rm"), &gone).unwrap();
    assert!(!gone.exists());
    let kept: PathBuf = new_file(dir.path(), "y.txt").unwrap();
    assert!(matches!(
        trash_with(Path::new("/usr/bin/false"), &kept),
        Err(FileOpError::Trash { .. })
    ));
    assert!(kept.exists());
}

fn right_click(vcx: &mut VisualTestContext, row: usize) {
    let selector: &'static str = if row == 0 {
        "explorer-row-0"
    } else {
        "explorer-row-1"
    };
    let at: gpui::Point<gpui::Pixels> = vcx.debug_bounds(selector).unwrap().center();
    vcx.simulate_mouse_down(at, MouseButton::Right, Modifiers::default());
    vcx.simulate_mouse_up(at, MouseButton::Right, Modifiers::default());
    vcx.run_until_parked();
}

fn click(vcx: &mut VisualTestContext, selector: &'static str) {
    let at: gpui::Point<gpui::Pixels> = vcx.debug_bounds(selector).unwrap().center();
    vcx.simulate_click(at, Modifiers::default());
    vcx.run_until_parked();
}

fn tabs(rig: &Rig, vcx: &mut VisualTestContext) -> Vec<PathBuf> {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs
            .items()
            .iter()
            .map(|t: &crate::editor::Tab| t.path.clone())
            .collect()
    })
}

fn status(rig: &Rig, vcx: &mut VisualTestContext) -> String {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.status.clone().unwrap_or_default()
    })
}

#[gpui::test]
fn the_root_row_offers_new_only_and_a_file_row_all_four(cx: &mut TestAppContext) {
    let (_rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    right_click(vcx, 0);
    assert!(vcx.debug_bounds("file-menu-new-file").is_some());
    assert!(
        vcx.debug_bounds("file-menu-rename").is_none(),
        "the root is not renamed here"
    );
    click(vcx, "editor-area");
    assert!(
        vcx.debug_bounds("file-menu").is_none(),
        "a press outside closes it"
    );
    right_click(vcx, 1);
    for row in [
        "file-menu-new-file",
        "file-menu-new-folder",
        "file-menu-rename",
        "file-menu-delete",
    ] {
        assert!(vcx.debug_bounds(row).is_some(), "{row}");
    }
}

#[gpui::test]
fn new_file_makes_it_in_the_folder_and_opens_it(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    right_click(vcx, 1);
    click(vcx, "file-menu-new-file");
    vcx.simulate_input("C.cs");
    vcx.simulate_keystrokes("enter");
    vcx.run_until_parked();
    let made: PathBuf = rig.root.join("C.cs");
    assert!(made.is_file(), "made next to A.cs");
    assert!(tabs(&rig, vcx).contains(&made), "and opened");
}

#[gpui::test]
fn rename_moves_the_file_and_its_tab(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    right_click(vcx, 1);
    click(vcx, "file-menu-rename");
    vcx.simulate_keystrokes("backspace backspace backspace backspace");
    vcx.simulate_input("Z.cs");
    vcx.simulate_keystrokes("enter");
    vcx.run_until_parked();
    let to: PathBuf = rig.root.join("Z.cs");
    assert!(to.is_file() && !rig.file.exists(), "{}", status(&rig, vcx));
    assert_eq!(tabs(&rig, vcx), vec![to], "the tab follows");
}

#[gpui::test]
fn delete_asks_then_trashes_and_closes_the_tab(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    right_click(vcx, 1);
    click(vcx, "file-menu-delete");
    assert!(rig.file.exists(), "nothing before the answer");
    vcx.simulate_prompt_answer("Move to Trash");
    vcx.run_until_parked();
    assert!(!rig.file.exists(), "{}", status(&rig, vcx));
    assert!(tabs(&rig, vcx).is_empty(), "its tab closed");
}

#[gpui::test]
fn a_file_with_unsaved_changes_is_not_renamed_and_no_write_refuses(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, _: &mut gpui::Context<ExplorerView>| {
            v.tabs.active_mut().unwrap().insert("x").unwrap();
        },
    );
    right_click(vcx, 1);
    click(vcx, "file-menu-rename");
    vcx.simulate_input("2");
    vcx.simulate_keystrokes("enter");
    vcx.run_until_parked();
    assert!(rig.file.exists(), "kept");
    assert!(
        status(&rig, vcx).contains("unsaved changes"),
        "{}",
        status(&rig, vcx)
    );
}

#[gpui::test]
fn no_write_refuses_every_change(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, true, true, "class A {}\n");
    right_click(vcx, 1);
    click(vcx, "file-menu-new-file");
    let asked: bool = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.quick_open.is_some()
    });
    assert!(!asked, "no name box");
    assert!(
        status(&rig, vcx).contains("--no-write"),
        "{}",
        status(&rig, vcx)
    );
}
