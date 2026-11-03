use std::path::PathBuf;

use bb_explorer::{Entry, Explorer, ExplorerError};

pub(crate) fn right(tree: &mut Explorer, selected: usize) -> Result<usize, ExplorerError> {
    let rows: Vec<(usize, &Entry)> = tree.visible();
    let Some((depth, entry)): Option<&(usize, &Entry)> = rows.get(selected) else {
        return Ok(selected);
    };
    if !entry.expandable() {
        return Ok(selected);
    }
    if !entry.expanded() {
        let path: PathBuf = entry.path().to_path_buf();
        tree.toggle(&path)?;
        return Ok(selected);
    }
    let next: Option<&(usize, &Entry)> = rows.get(selected.saturating_add(1));
    Ok(next
        .filter(|(level, _): &&(usize, &Entry)| level > depth)
        .map_or(selected, |_: &(usize, &Entry)| selected.saturating_add(1)))
}

pub(crate) fn left(tree: &mut Explorer, selected: usize) -> Result<usize, ExplorerError> {
    let rows: Vec<(usize, &Entry)> = tree.visible();
    let Some((depth, entry)): Option<&(usize, &Entry)> = rows.get(selected) else {
        return Ok(selected);
    };
    if entry.expanded() {
        let path: PathBuf = entry.path().to_path_buf();
        tree.toggle(&path)?;
        return Ok(selected);
    }
    if *depth == 0 {
        return Ok(selected);
    }
    Ok(rows
        .iter()
        .take(selected)
        .rposition(|(level, _): &(usize, &Entry)| level < depth)
        .unwrap_or(selected))
}

#[cfg(test)]
mod tests {
    use super::{left, right};
    use bb_explorer::Explorer;
    use std::fs;

    #[test]
    fn right_enters_first_child_of_expanded_folder() {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("Core")).unwrap();
        fs::write(dir.path().join("Core/Foo.cs"), "").unwrap();
        let mut tree: Explorer = Explorer::open(dir.path()).unwrap();
        assert_eq!(right(&mut tree, 1).unwrap(), 1);
        assert_eq!(right(&mut tree, 1).unwrap(), 2);
        assert_eq!(left(&mut tree, 2).unwrap(), 1);
    }
}

/// Select the preceding Explorer row.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct MoveUp;
/// Select the next Explorer row.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct MoveDown;
/// Collapse the selected folder or select its parent.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct MoveLeft;
/// Expand the selected folder.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct MoveRight;
/// Open the selected entry.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct Open;

/// The side panel. On the Files side the tree first follows the active
/// file (BB-BUG-029), as VS Code's Explorer does.
pub(crate) fn side_panel(
    view: &mut crate::ExplorerView,
    cx: &mut gpui::Context<crate::ExplorerView>,
) -> gpui::AnyElement {
    match view.tests.side {
        crate::tests_panel::Side::Files => {
            follow_active(view);
            gpui::IntoElement::into_any_element(crate::panel::explorer_panel(view, cx))
        }
        crate::tests_panel::Side::Tests => {
            gpui::IntoElement::into_any_element(crate::tests_panel::panel(view, cx))
        }
    }
}

/// Once per newly active file: open the folders down to it, select its row
/// and scroll it into view. A folder the user closes after that stays closed.
pub(crate) fn follow_active(view: &mut crate::ExplorerView) {
    let Some(path): Option<PathBuf> = view
        .tabs
        .active_tab()
        .map(|tab: &crate::editor::Tab| tab.path.clone())
    else {
        return;
    };
    if view.tabs.revealed.as_ref() == Some(&path) {
        return;
    }
    match view.tree.reveal(&path) {
        Ok(Some(row)) => {
            view.selected = row;
            view.explorer_scroll
                .scroll_to_item(row, gpui::ScrollStrategy::Nearest);
        }
        Ok(None) => {}
        Err(error) => view.report(&error),
    }
    view.tabs.revealed = Some(path);
}

#[cfg(test)]
mod follow_tests {
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
}
