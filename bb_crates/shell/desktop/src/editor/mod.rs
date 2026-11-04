mod bb_tabs;
mod input;
mod prompt;
mod view;

#[cfg(test)]
pub(crate) use view::pane_tests;

pub(crate) use bb_tabs::{
    apply_tab_menu, on_tab_actions, tab_bindings, tab_menu_item, TabMenuItem,
};
pub(crate) use bb_tabs::{go_back, go_forward, GoBack, GoForward, Tab, TabError, TabEvent, Tabs};
pub(crate) use bb_tabs::{Group, MAX_GROUPS};
pub(crate) use input::handle_key;
pub(crate) use input::link;
pub(crate) use prompt::Quit;
pub(crate) use view::{
    bottom_handle, build, handle, layout_panes, panes, status_bar, Columns, Edge, Lines, HANDLE_PX,
};

/// How often open files are checked against the disk (BB-BUG-035).
const DISK_CHECK: std::time::Duration = std::time::Duration::from_secs(2);

/// Every [`DISK_CHECK`], take the disk's text into each tab whose file
/// changed and that holds no unsaved edits, and new or removed files into
/// the Explorer, as VS Code does. Stops when the window goes.
pub(crate) fn watch_disk(window: &mut gpui::Window, cx: &mut gpui::Context<crate::ExplorerView>) {
    cx.spawn_in(
        window,
        async move |view: gpui::WeakEntity<crate::ExplorerView>,
                    cx: &mut gpui::AsyncWindowContext| loop {
            cx.background_executor().timer(DISK_CHECK).await;
            let alive: gpui::Result<()> = view.update_in(
                cx,
                |v: &mut crate::ExplorerView,
                 window: &mut gpui::Window,
                 cx: &mut gpui::Context<crate::ExplorerView>| {
                    reload_changed(v, window, cx);
                },
            );
            if alive.is_err() {
                return;
            }
        },
    )
    .detach();
}

/// Reload the changed clean tabs; reparse them and repaint.
pub(crate) fn reload_changed(
    view: &mut crate::ExplorerView,
    window: &mut gpui::Window,
    cx: &mut gpui::Context<crate::ExplorerView>,
) {
    let mut reloaded: Vec<std::path::PathBuf> = Vec::new();
    for tab in view.tabs.items_mut() {
        // An unreadable file keeps the buffer; the next save says why.
        if tab.document.reload_if_changed().unwrap_or(false) {
            tab.sync_edits();
            reloaded.push(tab.path.clone());
        }
    }
    for path in &reloaded {
        input::schedule_parse(view, path.clone(), window, cx);
    }
    if refresh_tree(view) || !reloaded.is_empty() {
        cx.notify();
    }
}

/// Take outside changes into the Explorer (BB-BUG-037); the selection
/// stays on its path. Returns whether the tree changed.
fn refresh_tree(view: &mut crate::ExplorerView) -> bool {
    let selected: Option<std::path::PathBuf> = view
        .tree
        .visible()
        .get(view.selected)
        .map(|(_, entry): &(usize, &bb_explorer::Entry)| entry.path().to_path_buf());
    if !view.tree.refresh().unwrap_or(false) {
        return false;
    }
    let rows: Vec<(usize, &bb_explorer::Entry)> = view.tree.visible();
    view.selected = selected
        .and_then(|path: std::path::PathBuf| {
            rows.iter()
                .position(|(_, entry): &(usize, &bb_explorer::Entry)| entry.path() == path)
        })
        .unwrap_or(view.selected)
        .min(rows.len().saturating_sub(1));
    true
}
