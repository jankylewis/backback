//! Script steps for live readback, mixed into `BB_KEYS` with the keys:
//! `wait:ready`, `wait:answer`, `sleep:<ms>`, `goto:<line>:<col>` (1-based),
//! `hover:<line>:<col>` (a real mouse move), `click:<line>:<col>` (a real
//! click; `cmdhover`/`cmdclick` hold Cmd; `clickpast:<line>` clicks right of
//! the line's end), `mark:<name>` (prints a line a capture script waits for, then
//! pauses), `state` (prints what the window shows), `testopen:<name>` /
//! `testrun:<name>` (the first spec whose name holds `name`: a row click /
//! its ▶), `teststop` (the ■), `open:<path>` (a file under the root), `tabmenu:<n>:<row>`
//! (tab `n`'s right-click menu row, e.g. `close-others`, `pin`). Keys and mouse go
//! through the window's own dispatch.

use std::time::{Duration, Instant};

use gpui::{AsyncWindowContext, Context, WeakEntity, Window};

mod bb_pointer;
mod state;
mod test_steps;

use crate::intel::model::LoadStatus;
use crate::ExplorerView;

/// Run one step; `false` when it is not a script step.
pub(crate) async fn run(
    step: &str,
    view: &WeakEntity<ExplorerView>,
    cx: &mut AsyncWindowContext,
) -> bool {
    let parts: Vec<&str> = step.split(':').collect();
    match parts.as_slice() {
        ["wait", "ready"] => wait(view, cx, ready).await,
        ["wait", "answer"] => wait(view, cx, |v: &ExplorerView| v.intel.in_flight.is_empty()).await,
        ["sleep", ms] => pause(cx, ms.parse::<u64>().unwrap_or(0)).await,
        ["goto", line, col] => goto(view, cx, (number(line), number(col))),
        ["hover", line, col] => bb_pointer::hover(view, cx, (number(line), number(col)), false),
        ["click", line, col] => bb_pointer::click(view, cx, (number(line), number(col)), false),
        ["cmdhover", line, col] => bb_pointer::hover(view, cx, (number(line), number(col)), true),
        ["cmdclick", line, col] => bb_pointer::click(view, cx, (number(line), number(col)), true),
        ["clickpast", line] => bb_pointer::click_past(view, cx, number(line)),
        ["mark", name] => {
            eprintln!("bb_mark {name}");
            pause(cx, 1_500).await;
        }
        ["state"] => state::state(view, cx),
        ["tests"] => tests(view, cx, false),
        ["tests", "expand"] => tests(view, cx, true),
        ["testopen", name] => test_steps::open(view, cx, name),
        ["testrun", name] => test_steps::run(view, cx, name),
        ["teststop"] => test_steps::stop(view, cx),
        ["open", relative] => open(view, cx, relative),
        ["tabmenu", n, row] => tab_menu(view, cx, number(n), row),
        _ => return false,
    }
    true
}

fn tests(view: &WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext, expand: bool) {
    let _gone: gpui::Result<()> = view.update_in(
        cx,
        |v: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>| {
            if expand {
                let path: Option<std::path::PathBuf> =
                    v.tests.model.projects().keys().next().cloned();
                if let Some(path) = path {
                    crate::tests_panel::toggle_project(v, path, window, cx);
                }
            } else {
                crate::tests_panel::open(v, window, cx);
            }
        },
    );
}

/// `tabmenu:<n>:<row>`: what that row of tab `n`'s menu does (`n` is
/// 1-based in the step; [`number`] made it 0-based).
fn tab_menu(view: &WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext, n: usize, row: &str) {
    let Some(item): Option<crate::editor::TabMenuItem> = crate::editor::tab_menu_item(row) else {
        eprintln!("bb_script no tab menu row {row}");
        return;
    };
    let _gone: gpui::Result<()> = view.update_in(
        cx,
        |v: &mut ExplorerView, _: &mut Window, cx: &mut Context<ExplorerView>| {
            crate::editor::apply_tab_menu(v, n, item);
            cx.notify();
        },
    );
}

fn ready(view: &ExplorerView) -> bool {
    view.intel.status == LoadStatus::Ready
}

/// `open:<path>`: open a file under the window's root, as a click does.
fn open(view: &WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext, relative: &str) {
    let _gone: gpui::Result<()> = view.update_in(
        cx,
        |v: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>| {
            let file: std::path::PathBuf = v.root.join(relative);
            v.open_file(&file, window, cx);
        },
    );
}

fn number(text: &str) -> usize {
    text.parse::<usize>().unwrap_or(1).saturating_sub(1)
}

async fn pause(cx: &mut AsyncWindowContext, ms: u64) {
    cx.background_executor()
        .timer(Duration::from_millis(ms))
        .await;
}

async fn wait(
    view: &WeakEntity<ExplorerView>,
    cx: &mut AsyncWindowContext,
    done: fn(&ExplorerView) -> bool,
) {
    let started: Instant = Instant::now();
    while started.elapsed() < Duration::from_secs(180) {
        let reached: bool = view
            .read_with(cx, |v: &ExplorerView, _: &gpui::App| done(v))
            .unwrap_or(true);
        if reached {
            eprintln!("bb_wait_ms={}", started.elapsed().as_millis());
            return;
        }
        pause(cx, 50).await;
    }
    eprintln!("bb_wait timeout");
}

fn goto(view: &WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext, (line, col): (usize, usize)) {
    let _gone: gpui::Result<()> = view.update(
        cx,
        |v: &mut ExplorerView, cx: &mut Context<ExplorerView>| {
            let target: Option<bb_text::CharIdx> = v
                .tabs
                .active()
                .and_then(|doc: &bb_text::Document| doc.line_start(line))
                .map(|start: bb_text::CharIdx| {
                    bb_text::CharIdx::new(start.raw().saturating_add(col))
                });
            if let (Some(doc), Some(at)) = (v.tabs.active_mut(), target) {
                if doc.set_cursor(at).is_ok() {
                    v.editor_scroll
                        .scroll_to_item(line, gpui::ScrollStrategy::Center);
                }
            }
            cx.notify();
        },
    );
}
