//! File > Open... (Cmd+O): the macOS open panel for a folder, a solution or
//! a file; the choice opens in a window and goes first in the recent list.
//! File > Start Page...: the recent workspaces and Open... (BBEOD23).

use gpui::{AnyView, AnyWindowHandle, App, KeyBinding, Menu, MenuItem, PathPromptOptions, Window};

mod bb_start_page;
mod start_entries;
mod start_script;
#[cfg(test)]
mod tests;

pub(crate) use bb_start_page::show as show_start_page;
pub(crate) use start_script::describe as start_state;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct OpenWorkspace;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct ShowStartPage;

/// App-wide handlers; the start page opens workspaces with `no_write`, the
/// launch's `--no-write`.
pub(crate) fn register(cx: &mut App, no_write: bool) {
    cx.on_action(|_: &OpenWorkspace, cx: &mut App| ask(cx, false, None));
    cx.on_action(move |_: &ShowStartPage, cx: &mut App| show_start_page(no_write, cx));
}

/// Show the open panel; open what the user picks in a new window, then close
/// `page`, the start page that asked, if any (a Cancel keeps it).
pub(crate) fn ask(cx: &mut App, no_write: bool, page: Option<AnyWindowHandle>) {
    let picked: std::pin::Pin<Box<dyn std::future::Future<Output = Option<std::path::PathBuf>>>> =
        Box::pin(first_path(cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: true,
            multiple: false,
            prompt: Some("Open".into()),
        })));
    cx.spawn(async move |cx: &mut gpui::AsyncApp| {
        let Some(path): Option<std::path::PathBuf> = picked.await else {
            return;
        };
        let start: crate::args::Start = crate::args::start_for(path, no_write);
        let opened: bool = cx.update(|cx: &mut App| super::open_workspace(&start, cx));
        if let (true, Some(page)) = (opened, page) {
            let _gone: gpui::Result<()> =
                page.update(cx, |_: AnyView, window: &mut Window, _: &mut App| {
                    window.remove_window();
                });
        }
    })
    .detach();
}

/// The first picked path; None on Cancel or a failed panel.
async fn first_path<F, A, B>(answer: F) -> Option<std::path::PathBuf>
where
    F: std::future::Future<Output = Result<Result<Option<Vec<std::path::PathBuf>>, A>, B>>,
{
    answer.await.ok()?.ok()??.into_iter().next()
}

pub(crate) fn bindings() -> [KeyBinding; 6] {
    [
        KeyBinding::new("cmd-o", OpenWorkspace, None),
        KeyBinding::new("cmd-shift-o", crate::quick_open::GoToFile, None),
        KeyBinding::new("cmd-[", crate::editor::GoBack, None),
        KeyBinding::new("cmd-]", crate::editor::GoForward, None),
        KeyBinding::new("cmd-f12", crate::quick_open::GoToSymbol, None),
        KeyBinding::new("cmd-shift-f", crate::quick_open::FindInFiles, None),
    ]
}

pub(crate) fn file_menu() -> Menu {
    Menu::new("File").items([
        MenuItem::action("Open...", OpenWorkspace),
        MenuItem::action("Start Page...", ShowStartPage),
        MenuItem::action("Go to File...", crate::quick_open::GoToFile),
        MenuItem::action("Go to Symbol...", crate::quick_open::GoToSymbol),
        MenuItem::action("Find in Files...", crate::quick_open::FindInFiles),
        MenuItem::action("Back", crate::editor::GoBack),
        MenuItem::action("Forward", crate::editor::GoForward),
    ])
}
