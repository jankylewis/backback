//! Delete: ask, then move the entry to the Trash off the UI thread.

use std::path::{Path, PathBuf};

use gpui::{AsyncWindowContext, Context, PromptLevel, WeakEntity, Window};

use super::{bb_ops, file_name, release};
use crate::ExplorerView;

/// Ask before a delete; on yes, move `path` to the Trash off the UI thread.
pub(super) fn ask_delete(path: PathBuf, window: &mut Window, cx: &mut Context<ExplorerView>) {
    let question: String = format!("Move \u{201c}{}\u{201d} to the Trash?", file_name(&path));
    on_yes(
        window.prompt(
            PromptLevel::Warning,
            &question,
            Some("You can put it back from the Trash."),
            &["Move to Trash", "Cancel"],
            cx,
        ),
        path,
        window,
        cx,
    );
}

/// When the prompt's first button (`Move to Trash`) is the answer, trash.
fn on_yes<E: 'static>(
    answer: impl std::future::Future<Output = Result<usize, E>> + 'static,
    path: PathBuf,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    cx.spawn_in(
        window,
        async move |view: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext| {
            if answer.await.ok() != Some(0) {
                return;
            }
            let _gone: gpui::Result<()> = view.update_in(
                cx,
                |v: &mut ExplorerView, _: &mut Window, cx: &mut Context<ExplorerView>| {
                    trash(v, path, cx);
                },
            );
        },
    )
    .detach();
}

fn trash(view: &mut ExplorerView, path: PathBuf, cx: &mut Context<ExplorerView>) {
    if let Err(why) = release(view, &path) {
        view.report(&why);
        cx.notify();
        return;
    }
    let moved: PathBuf = path.clone();
    let task: gpui::Task<Result<(), bb_ops::FileOpError>> = cx
        .background_executor()
        .spawn(async move { bb_ops::trash_with(Path::new(bb_ops::TRASH), &moved) });
    cx.spawn(
        async move |view: WeakEntity<ExplorerView>, cx: &mut gpui::AsyncApp| {
            let result: Result<(), bb_ops::FileOpError> = task.await;
            let _gone: gpui::Result<()> = view.update(
                cx,
                |v: &mut ExplorerView, cx: &mut Context<ExplorerView>| {
                    match result {
                        Ok(()) => {
                            let _changed: bool = v.tree.refresh().unwrap_or(false);
                            v.status = Some(format!("Moved {} to the Trash", file_name(&path)));
                        }
                        Err(why) => v.report(&why),
                    }
                    cx.notify();
                },
            );
        },
    )
    .detach();
}
