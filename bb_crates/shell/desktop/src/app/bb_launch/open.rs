//! File > Open... (Cmd+O): the macOS open panel for a folder, a solution or
//! a file; the choice opens in a window and goes first in the recent list.

use gpui::{App, KeyBinding, Menu, MenuItem, PathPromptOptions};

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct OpenWorkspace;

pub(crate) fn register(cx: &mut App) {
    cx.on_action(|_: &OpenWorkspace, cx: &mut App| ask(cx, false));
}

/// Show the open panel; open what the user picks, in a new window.
pub(crate) fn ask(cx: &mut App, no_write: bool) {
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
        cx.update(|cx: &mut App| super::open_workspace(&start, cx));
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

pub(crate) fn bindings() -> [KeyBinding; 5] {
    [
        KeyBinding::new("cmd-o", OpenWorkspace, None),
        KeyBinding::new("cmd-shift-o", crate::quick_open::GoToFile, None),
        KeyBinding::new("cmd-[", crate::editor::GoBack, None),
        KeyBinding::new("cmd-]", crate::editor::GoForward, None),
        KeyBinding::new("cmd-f12", crate::quick_open::GoToSymbol, None),
    ]
}

pub(crate) fn file_menu() -> Menu {
    Menu::new("File").items([
        MenuItem::action("Open...", OpenWorkspace),
        MenuItem::action("Go to File...", crate::quick_open::GoToFile),
        MenuItem::action("Go to Symbol...", crate::quick_open::GoToSymbol),
        MenuItem::action("Back", crate::editor::GoBack),
        MenuItem::action("Forward", crate::editor::GoForward),
    ])
}
