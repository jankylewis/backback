//! The readback runner on the start page (BBEOD23). A workspace window
//! starts `BB_KEYS` once a file opens; the page has no file, so it runs the
//! steps itself: `sleep:<ms>`, `state` (prints `bb_start_state`) and keys
//! through the page window's own dispatch. A step that opens a workspace
//! closes the page; the rest of the steps then run in that new window.

use std::time::Duration;

use gpui::{AnyView, AnyWindowHandle, App, AsyncApp, Window, WindowHandle};

use super::bb_start_page::StartPage;
use crate::ExplorerView;

/// The `bb_start_state` line: whether a start page is shown, the selected
/// entry and each entry.
pub(crate) fn describe(cx: &App) -> String {
    let page: Option<&StartPage> = cx
        .windows()
        .into_iter()
        .find_map(|window: AnyWindowHandle| window.downcast::<StartPage>())
        .and_then(|page: WindowHandle<StartPage>| page.read(cx).ok());
    match page {
        Some(page) => format!(
            "bb_start_state shown=true selected={} entries={:?}",
            page.selected(),
            page.labels()
        ),
        None => String::from("bb_start_state shown=false selected=0 entries=[]"),
    }
}

/// Run `BB_KEYS` on a page that just opened; once per process.
pub(super) fn drive(page: WindowHandle<StartPage>, cx: &mut App) {
    let steps: Vec<String> = crate::latency::script_keys();
    if steps.is_empty() || !crate::latency::claim() {
        return;
    }
    cx.spawn(async move |cx: &mut AsyncApp| run(page, steps, cx).await)
        .detach();
}

async fn run(page: WindowHandle<StartPage>, steps: Vec<String>, cx: &mut AsyncApp) {
    let mut rest: std::vec::IntoIter<String> = steps.into_iter();
    while let Some(step) = rest.next() {
        let before: Vec<AnyWindowHandle> = cx.update(|cx: &mut App| cx.windows());
        one_step(&step, page.into(), cx).await;
        if page.read_with(cx, |_: &StartPage, _: &App| ()).is_err() {
            hand_over(rest.collect(), &before, cx);
            return;
        }
    }
    cx.update(crate::latency::quit_when_asked);
}

async fn one_step(step: &str, page: AnyWindowHandle, cx: &mut AsyncApp) {
    if step == "state" {
        let line: String = cx.update(|cx: &mut App| describe(cx));
        eprintln!("{line}");
    } else if let Some(ms) = step.strip_prefix("sleep:") {
        pause(cx, ms.parse::<u64>().unwrap_or(0)).await;
    } else {
        send_key(step, page, cx);
        pause(cx, 30).await;
    }
}

fn send_key(key: &str, page: AnyWindowHandle, cx: &mut AsyncApp) {
    let Ok(stroke): Result<gpui::Keystroke, _> =
        gpui::Keystroke::parse_with_os(key, gpui::OperatingSystem::Mac)
    else {
        eprintln!("bb_keys bad_key={key}");
        return;
    };
    let _gone: gpui::Result<bool> = page
        .update(cx, |_: AnyView, window: &mut Window, cx: &mut App| {
            window.dispatch_keystroke(stroke, cx)
        });
}

/// The page closed: the rest runs in the workspace window that step opened.
fn hand_over(rest: Vec<String>, before: &[AnyWindowHandle], cx: &mut AsyncApp) {
    let opened: Option<WindowHandle<ExplorerView>> = cx.update(|cx: &mut App| {
        cx.windows()
            .into_iter()
            .filter(|window: &AnyWindowHandle| !before.contains(window))
            .find_map(|window: AnyWindowHandle| window.downcast::<ExplorerView>())
    });
    let Some(workspace): Option<WindowHandle<ExplorerView>> = opened else {
        eprintln!("bb_start_state closed and no workspace opened");
        cx.update(crate::latency::quit_when_asked);
        return;
    };
    let _gone: gpui::Result<()> = workspace.update(
        cx,
        |_: &mut ExplorerView, window: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
            crate::latency::run_keys(rest, window, cx);
        },
    );
}

async fn pause(cx: &mut AsyncApp, ms: u64) {
    cx.background_executor()
        .timer(Duration::from_millis(ms))
        .await;
}
