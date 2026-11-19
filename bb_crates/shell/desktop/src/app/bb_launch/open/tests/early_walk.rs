//! BB-PRF-007: a command-line launch walks its folder for languages off the
//! UI thread, once, and the window shows what that walk found.

use std::path::PathBuf;
use std::thread::ThreadId;

use bb_lang::fixtures::FIXTURES;
use bb_lang::{Lang, LangSet, Langs};
use gpui::{AnyWindowHandle, App, TestAppContext, WindowHandle};

use crate::launch::langs::{walk_early, walks_of};
use crate::launch::recent::test_store;
use crate::ExplorerView;

fn shown_langs(cx: &mut TestAppContext) -> Option<Langs> {
    let window: WindowHandle<ExplorerView> = cx
        .windows()
        .into_iter()
        .filter_map(|w: AnyWindowHandle| w.downcast::<ExplorerView>())
        .next_back()?;
    window
        .read_with(cx, |view: &ExplorerView, _: &App| view.langs.langs)
        .ok()
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

#[gpui::test]
fn a_launch_walks_its_folder_once_off_the_ui_thread_and_the_window_shows_what_it_found(
    cx: &mut TestAppContext,
) {
    let store: tempfile::TempDir = tempfile::tempdir().unwrap();
    test_store(Some(store.path().to_path_buf()));
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = FIXTURES[4].write(base.path()).unwrap();
    let start: crate::args::Start = crate::args::start_for(root, false);
    walk_early(&start.root, start.ticked);
    let opened: bool = cx.update(|cx: &mut App| crate::launch::open_workspace(&start, cx));
    cx.run_until_parked();
    let shown: Option<Langs> = shown_langs(cx);
    let walkers: Vec<ThreadId> = walks_of(&start.root);
    close_all(cx);
    test_store(None);
    let held: LangSet = LangSet::of(&[Lang::CSharp, Lang::Python]);
    assert!(opened);
    assert_eq!(shown, Some(Langs::new(held, held)));
    assert_eq!(walkers.len(), 1, "one walk of the folder: {walkers:?}");
    assert_ne!(
        walkers[0],
        std::thread::current().id(),
        "the folder was walked on the UI thread"
    );
}
