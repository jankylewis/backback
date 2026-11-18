//! Opening the start page: show one, a page for the recent list, and the
//! page File > Languages... opens for its workspace (BBEOD26).

use std::path::PathBuf;

use gpui::{
    prelude::*, px, size, AnyWindowHandle, App, Bounds, Context, FocusHandle, TitlebarOptions,
    Window, WindowBounds, WindowHandle, WindowOptions,
};

use super::StartPage;

/// Show the page: bring an open one to the front, else open one.
pub(crate) fn show(no_write: bool, cx: &mut App) {
    let shown: Option<WindowHandle<StartPage>> = cx
        .windows()
        .into_iter()
        .find_map(|window: AnyWindowHandle| window.downcast::<StartPage>());
    if let Some(page) = shown {
        let _gone: gpui::Result<()> = page.update(
            cx,
            |_: &mut StartPage, window: &mut Window, _: &mut Context<StartPage>| {
                window.activate_window();
            },
        );
        return;
    }
    if let Some(page) = open(crate::launch::recent::list(), no_write, cx) {
        super::script::drive(page, cx);
    }
}

/// File > Languages... (BBEOD26): a page with `root` first and its boxes
/// at `ticks`; its Open closes `replace` and reopens `root` with the boxes.
pub(crate) fn reopen(
    root: &std::path::Path,
    ticks: bb_lang::LangSet,
    replace: AnyWindowHandle,
    cx: &mut App,
) {
    let mut recent: Vec<PathBuf> = crate::launch::recent::list();
    recent.retain(|path: &PathBuf| path != root);
    recent.insert(0, root.to_path_buf());
    let Some(page): Option<WindowHandle<StartPage>> = open(recent, false, cx) else {
        return;
    };
    let _gone: gpui::Result<()> = page.update(
        cx,
        |page: &mut StartPage, _: &mut Window, _: &mut Context<StartPage>| {
            page.ticks = ticks;
            page.replace = Some(replace);
        },
    );
}

/// Close `window` (File > Languages... reopens it); its own guard has
/// already found no unsaved tab.
pub(super) fn close(window: AnyWindowHandle, cx: &mut App) {
    let _gone: gpui::Result<()> =
        window.update(cx, |_: gpui::AnyView, window: &mut Window, _: &mut App| {
            window.remove_window();
        });
}

/// A page for `recent` in a new window, focused.
pub(crate) fn open(
    recent: Vec<PathBuf>,
    no_write: bool,
    cx: &mut App,
) -> Option<WindowHandle<StartPage>> {
    let bounds: Bounds<gpui::Pixels> = Bounds::centered(None, size(px(640.0), px(640.0)), cx);
    let opened: gpui::Result<WindowHandle<StartPage>> = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("Welcome to backback".into()),
                ..Default::default()
            }),
            ..Default::default()
        },
        |window: &mut Window, cx: &mut App| {
            let page: gpui::Entity<StartPage> =
                cx.new(|cx: &mut Context<StartPage>| StartPage::new(recent, no_write, window, cx));
            let focus: FocusHandle = page.read(cx).focus.clone();
            window.focus(&focus, cx);
            page
        },
    );
    match opened {
        Ok(page) => Some(page),
        Err(error) => {
            eprintln!("backback: cannot open the start page: {error}");
            None
        }
    }
}
