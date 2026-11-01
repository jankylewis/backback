//! Script steps for live readback, mixed into `BB_KEYS` with the keys:
//! `wait:ready`, `wait:answer`, `sleep:<ms>`, `goto:<line>:<col>` (1-based),
//! `hover:<line>:<col>` (a real mouse move), `mark:<name>` (prints a line a
//! capture script waits for, then pauses), `state` (prints what the window
//! shows). Keys and mouse go through the window's own dispatch.

use std::time::{Duration, Instant};

use gpui::{AsyncWindowContext, Context, MouseMoveEvent, PlatformInput, WeakEntity, Window};

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
        ["wait", "answer"] => wait(view, cx, answered).await,
        ["sleep", ms] => pause(cx, ms.parse::<u64>().unwrap_or(0)).await,
        ["goto", line, col] => goto(view, cx, (number(line), number(col))),
        ["hover", line, col] => hover(view, cx, (number(line), number(col))),
        ["mark", name] => {
            eprintln!("bb_mark {name}");
            pause(cx, 1_500).await;
        }
        ["state"] => state(view, cx),
        ["tests"] => tests(view, cx, false),
        ["tests", "expand"] => tests(view, cx, true),
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

fn ready(view: &ExplorerView) -> bool {
    view.intel.status == LoadStatus::Ready
}

fn answered(view: &ExplorerView) -> bool {
    view.intel.in_flight.is_empty()
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

/// Move the mouse to the middle of char `col` on `line`, as the window draws it.
fn hover(
    view: &WeakEntity<ExplorerView>,
    cx: &mut AsyncWindowContext,
    (line, col): (usize, usize),
) {
    let _gone: gpui::Result<()> = cx.update(|window: &mut Window, cx: &mut gpui::App| {
        let Some(view): Option<gpui::Entity<ExplorerView>> = view.upgrade() else {
            return;
        };
        let (left, top, scrolled): (f32, f32, f32) =
            view.read_with(cx, |v: &ExplorerView, _: &gpui::App| {
                let state: std::cell::Ref<'_, gpui::UniformListScrollState> =
                    v.editor_scroll.0.borrow();
                let bounds: gpui::Bounds<gpui::Pixels> = state.base_handle.bounds();
                (
                    v.panel_width + 56.0,
                    f32::from(bounds.origin.y),
                    f32::from(state.base_handle.offset().y),
                )
            });
        let row: f32 = f32::from(u16::try_from(line).unwrap_or(u16::MAX));
        let column: f32 = f32::from(u16::try_from(col).unwrap_or(u16::MAX));
        let position: gpui::Point<gpui::Pixels> = gpui::point(
            gpui::px(left + 7.8 * column + 3.9),
            gpui::px(top + scrolled + 22.0 * row + 11.0),
        );
        window.dispatch_event(
            PlatformInput::MouseMove(MouseMoveEvent {
                position,
                pressed_button: None,
                modifiers: gpui::Modifiers::default(),
            }),
            cx,
        );
    });
}

fn state(view: &WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext) {
    let _gone: gpui::Result<()> = view.read_with(cx, |v: &ExplorerView, _: &gpui::App| {
        let tab: Option<&crate::editor::Tab> = v.tabs.active_tab();
        let hover: String = v
            .intel
            .hover
            .as_ref()
            .and_then(|h: &crate::intel::model::popups::HoverPopup| h.text.lines().next())
            .unwrap_or_default()
            .to_owned();
        eprintln!(
            "bb_state status={:?} tab={:?} line={} read_only={} completion={:?} hover={hover:?} refs={:?} rename={:?} actions={:?} problems={} message={:?}",
            v.intel.status.label(),
            tab.map(|t: &crate::editor::Tab| t.path.display().to_string()),
            tab.map_or(0, |t: &crate::editor::Tab| t.document.cursor_line().saturating_add(1)),
            tab.is_some_and(|t: &crate::editor::Tab| t.read_only),
            v.intel.completion.as_ref().map(|c: &crate::intel::model::popups::CompletionPopup| c.visible().len()),
            v.intel.references.as_ref().map(|r: &crate::intel::model::popups::PlacesPanel| r.places.len()),
            v.intel.rename_preview.as_ref().map(crate::intel::model::popups::RenamePreview::counts),
            v.intel.actions.as_ref().map(|a: &crate::intel::model::popups::ActionsPopup| a.items.len()),
            v.intel.problems().len(),
            v.status,
        );
        let discovered: usize = v.tests.model.projects().values()
            .map(|project: &crate::tests_panel::model::ProjectTests| project.specs.len()).sum();
        let loading: usize = v.tests.model.projects().values()
            .filter(|project: &&crate::tests_panel::model::ProjectTests| project.loading).count();
        eprintln!("bb_tests_state projects={} discovered={discovered} loading={loading}",
            v.tests.model.projects().len());
    });
}
