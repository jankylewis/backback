mod bb_tabs;
mod input;
mod prompt;
mod view;

pub(crate) use bb_tabs::{go_back, go_forward, GoBack, GoForward, Tab, TabError, TabEvent, Tabs};
pub(crate) use input::handle_key;
pub(crate) use input::link;
pub(crate) use prompt::Quit;
pub(crate) use view::{editor_body, status_bar, tab_bar, Columns};

/// How often open files are checked against the disk (BB-BUG-035).
const DISK_CHECK: std::time::Duration = std::time::Duration::from_secs(2);

/// Every [`DISK_CHECK`], take the disk's text into each tab whose file
/// changed and that holds no unsaved edits, as VS Code does; one `stat`
/// per open file. Stops when the window goes.
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
    if !reloaded.is_empty() {
        cx.notify();
    }
}
