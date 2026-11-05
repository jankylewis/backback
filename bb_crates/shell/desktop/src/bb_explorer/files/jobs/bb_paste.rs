//! Paste (BB-BUG-052): the last Copy into a folder, off the UI thread, under
//! a free name; the tree then shows and selects the new entry.

use std::path::PathBuf;

use gpui::{Context, WeakEntity};

use super::{bb_ops, file_name, show};
use crate::ExplorerView;

/// Copy the last Copy into `folder`; the result shows when it is done.
pub(crate) fn paste(view: &mut ExplorerView, folder: PathBuf, cx: &mut Context<ExplorerView>) {
    let Some(from): Option<PathBuf> = view.file_menu.copied.clone() else {
        return;
    };
    view.status = Some(format!("Pasting {}...", file_name(&from)));
    let task: gpui::Task<Result<PathBuf, bb_ops::FileOpError>> = cx
        .background_executor()
        .spawn(async move { bb_ops::copy_into(&from, &folder) });
    cx.spawn(
        async move |view: WeakEntity<ExplorerView>, cx: &mut gpui::AsyncApp| {
            let result: Result<PathBuf, bb_ops::FileOpError> = task.await;
            let _gone: gpui::Result<()> = view.update(
                cx,
                |v: &mut ExplorerView, cx: &mut Context<ExplorerView>| {
                    match result {
                        Ok(to) => {
                            show(v, &to, cx);
                            v.status = Some(format!("Pasted {}", file_name(&to)));
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
