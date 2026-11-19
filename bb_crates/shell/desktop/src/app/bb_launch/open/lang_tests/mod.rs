//! BBEOD26 in real windows: every tick set on every fixture folder opens a
//! window that starts only its active languages' tools, colours only their
//! files and shows Debug only with C#; the start page's boxes work by mouse
//! and keys, refuse to open with none ticked, and come back on reopen.
//! BBEOD27/28: Rust and Python requests, explorers, Build and Tests
//! (`tools`).

use std::path::Path;

use bb_lang::LangSet;
use gpui::{AnyWindowHandle, App, TestAppContext, WindowHandle};

use crate::ExplorerView;

mod bb_matrix;
mod page;
pub(crate) mod tools;
pub(crate) mod watchdog;

fn opened(cx: &mut TestAppContext) -> Vec<WindowHandle<ExplorerView>> {
    cx.windows()
        .into_iter()
        .filter_map(|w: AnyWindowHandle| w.downcast::<ExplorerView>())
        .collect()
}

fn close_all(cx: &mut TestAppContext) {
    for window in cx.windows() {
        let _gone: gpui::Result<()> =
            window.update(cx, |_: gpui::AnyView, w: &mut gpui::Window, _: &mut App| {
                w.remove_window();
            });
    }
    cx.run_until_parked();
}

fn open_with(cx: &mut TestAppContext, root: &Path, ticked: LangSet) -> WindowHandle<ExplorerView> {
    let mut start: crate::args::Start = crate::args::start_for(root.to_path_buf(), false);
    start.ticked = Some(ticked);
    assert!(cx.update(|cx: &mut App| crate::launch::open_workspace(&start, cx)));
    cx.run_until_parked();
    opened(cx).pop().unwrap()
}
