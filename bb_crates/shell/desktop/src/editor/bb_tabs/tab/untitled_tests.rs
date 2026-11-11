//! Cmd+N through the window: a "New file" tab in the focused file's folder;
//! Cmd+S asks where to save it, starting there; a close asks too, and a
//! Cancel in that dialog keeps the tab and stops the close.

use std::path::PathBuf;

use gpui::{TestAppContext, VisualTestContext, Window};

use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
use crate::ExplorerView;

/// (name, untitled, dirty) of the active tab.
fn active(rig: &Rig, vcx: &mut VisualTestContext) -> (String, bool, bool) {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        let tab: &crate::editor::Tab = v.tabs.active_tab().unwrap();
        let name: String = tab.path.file_name().unwrap().to_string_lossy().into_owned();
        (name, tab.is_untitled(), tab.document.is_dirty())
    })
}

fn new_file(rig: &Rig, vcx: &mut VisualTestContext) {
    rig.view.update_in(
        vcx,
        |v: &mut ExplorerView, window: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
            super::on_new_file(v, &super::NewFile, window, cx);
        },
    );
    vcx.run_until_parked();
}

#[gpui::test]
fn cmd_n_opens_a_new_file_that_cmd_s_saves_where_the_user_picks(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    vcx.update(|_: &mut Window, cx: &mut gpui::App| crate::launch::bind_keys(cx));
    vcx.simulate_keystrokes("cmd-n");
    vcx.run_until_parked();
    assert_eq!(active(&rig, vcx), ("New file".to_owned(), true, false));
    new_file(&rig, vcx);
    assert_eq!(
        active(&rig, vcx).0,
        "New file 2",
        "a second one is numbered"
    );
    vcx.simulate_keystrokes("x");
    vcx.simulate_keystrokes("cmd-s");
    vcx.run_until_parked();
    assert!(vcx.did_prompt_for_new_path(), "Cmd+S asks where to save");
    let root: PathBuf = rig.root.clone();
    let target: PathBuf = root.join("Saved.cs");
    let picked: PathBuf = target.clone();
    vcx.simulate_new_path_selection(move |dir: &std::path::Path| {
        assert_eq!(dir, root, "the dialog starts in the focused file's folder");
        Some(picked)
    });
    vcx.run_until_parked();
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "x");
    assert_eq!(active(&rig, vcx), ("Saved.cs".to_owned(), false, false));
}

/// The close prompt's Save on the new file at `path`.
fn save_and_close(rig: &Rig, vcx: &mut VisualTestContext, path: PathBuf) {
    rig.view.update_in(
        vcx,
        |v: &mut ExplorerView, window: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
            super::save_as(v, path, true, window, cx);
        },
    );
    vcx.run_until_parked();
}

#[gpui::test]
fn closing_an_edited_new_file_asks_where_and_cancel_keeps_it(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    vcx.update(|_: &mut Window, cx: &mut gpui::App| crate::launch::bind_keys(cx));
    new_file(&rig, vcx);
    vcx.simulate_keystrokes("y");
    let path: PathBuf = rig.view.update(
        vcx,
        |v: &mut ExplorerView, _: &mut gpui::Context<ExplorerView>| {
            assert!(!v.tabs.close_active(), "an edited file asks first");
            v.tabs.asking().unwrap().to_path_buf()
        },
    );
    save_and_close(&rig, vcx, path.clone());
    vcx.simulate_new_path_selection(|_: &std::path::Path| None);
    vcx.run_until_parked();
    let (kept, asking): (bool, bool) =
        rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
            (
                v.tabs
                    .active_tab()
                    .is_some_and(|t: &crate::editor::Tab| t.is_untitled()),
                v.tabs.pending_close(),
            )
        });
    assert!(
        kept && !asking,
        "Cancel keeps the new file and ends the close"
    );
    let target: PathBuf = rig.root.join("Kept.cs");
    let picked: PathBuf = target.clone();
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, _: &mut gpui::Context<ExplorerView>| {
            assert!(!v.tabs.close_active());
        },
    );
    save_and_close(&rig, vcx, path);
    vcx.simulate_new_path_selection(move |_: &std::path::Path| Some(picked));
    vcx.run_until_parked();
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "y");
    let open: usize = rig
        .view
        .read_with(vcx, |v: &ExplorerView, _: &gpui::App| v.tabs.items().len());
    assert_eq!(open, 1, "saved, then closed: only A.cs is left");
}
