//! BB-BUG-029 and BB-BUG-060: the Explorer follows the active file.
use gpui::{Entity, TestAppContext, VisualTestContext, Window, WindowAppearance};
use std::path::PathBuf;

use crate::ExplorerView;

/// BB-BUG-029: opening a deep file opens its folders in the Explorer and
/// selects its row; a folder closed after that stays closed.
#[gpui::test]
fn the_explorer_selects_the_active_file(cx: &mut TestAppContext) {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = dir.path().canonicalize().unwrap();
    std::fs::create_dir_all(root.join("MbAx/agentmates")).unwrap();
    let file: PathBuf = root.join("MbAx/agentmates/walk.py");
    std::fs::write(&file, "x = 1\n").unwrap();
    let tree: bb_explorer::Explorer = bb_explorer::Explorer::open(&root).unwrap();
    let start: PathBuf = root.clone();
    let (view, vcx): (Entity<ExplorerView>, &mut VisualTestContext) =
        cx.add_window_view(|_: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
            crate::launch::fresh_view(tree, (start, false), WindowAppearance::Dark, cx)
        });
    vcx.run_until_parked();
    let target: PathBuf = file.clone();
    view.update_in(
        vcx,
        |v: &mut ExplorerView, window: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
            v.open_file(&target, window, cx);
        },
    );
    vcx.run_until_parked();
    let selected: Option<PathBuf> = view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tree
            .visible()
            .get(v.selected)
            .map(|(_, entry): &(usize, &bb_explorer::Entry)| entry.path().to_path_buf())
    });
    assert_eq!(
        selected,
        Some(file.clone()),
        "the active file's row is selected"
    );
    let folder: PathBuf = root.join("MbAx");
    view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            v.tree.toggle(&folder).unwrap();
            cx.notify();
        },
    );
    vcx.run_until_parked();
    let shown: bool = view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tree
            .visible()
            .iter()
            .any(|(_, entry): &(usize, &bb_explorer::Entry)| entry.path() == file.as_path())
    });
    assert!(!shown, "the user closed MbAx: it stays closed");
}

/// BB-BUG-060: a file far down a tall tree is scrolled into sight when
/// it becomes active, not only selected, as VS Code's Explorer does.
#[gpui::test]
fn the_active_files_row_is_scrolled_into_the_explorers_view(cx: &mut TestAppContext) {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = dir.path().canonicalize().unwrap();
    for n in 0..120 {
        std::fs::create_dir_all(root.join(format!("f{n:03}"))).unwrap();
    }
    let near: PathBuf = root.join("f000/Near.cs");
    let far: PathBuf = root.join("f119/Deep/Far.cs");
    std::fs::create_dir_all(far.parent().unwrap()).unwrap();
    std::fs::write(&near, "class Near {}\n").unwrap();
    std::fs::write(&far, "class Far {}\n").unwrap();
    let tree: bb_explorer::Explorer = bb_explorer::Explorer::open(&root).unwrap();
    let start: PathBuf = root.clone();
    let (view, vcx): (Entity<ExplorerView>, &mut VisualTestContext) =
        cx.add_window_view(|_: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
            crate::launch::fresh_view(tree, (start, false), WindowAppearance::Dark, cx)
        });
    for file in [&near, &far, &near, &far] {
        let target: PathBuf = file.clone();
        view.update_in(
            vcx,
            |v: &mut ExplorerView, window: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
                v.open_file(&target, window, cx);
            },
        );
        vcx.run_until_parked();
        assert_in_sight(&view, vcx, &target);
    }
    // The user scrolls the tree away and works in it, then clicks back
    // into the file, or on its own tab (tab 1, Far.cs): the tree comes back.
    for place in ["editor-area", "tab-1"] {
        scroll_the_tree_away(&view, vcx);
        let at: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds(place).unwrap();
        vcx.simulate_click(at.center(), gpui::Modifiers::default());
        vcx.run_until_parked();
        assert_in_sight(&view, vcx, &far);
    }
}

/// The tree scrolled to its top and focused, as a user browsing it.
fn scroll_the_tree_away(view: &Entity<ExplorerView>, vcx: &mut VisualTestContext) {
    view.update_in(
        vcx,
        |v: &mut ExplorerView, window: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
            v.explorer_scroll
                .scroll_to_item(0, gpui::ScrollStrategy::Top);
            window.focus(&v.focus, cx);
        },
    );
    vcx.run_until_parked();
}

fn assert_in_sight(
    view: &Entity<ExplorerView>,
    vcx: &mut VisualTestContext,
    file: &std::path::Path,
) {
    let row: usize = view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| v.selected);
    let list: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds("explorer-panel").unwrap();
    let shown: Option<gpui::Bounds<gpui::Pixels>> =
        vcx.debug_bounds(Box::leak(format!("explorer-row-{row}").into_boxed_str()));
    assert!(
        shown.is_some_and(|b: gpui::Bounds<gpui::Pixels>| {
            b.top() >= list.top() && b.bottom() <= list.bottom()
        }),
        "{} at row {row} is out of sight: {shown:?} in {list:?}",
        file.display()
    );
}
