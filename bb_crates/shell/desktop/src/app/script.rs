//! Script steps for live readback, mixed into `BB_KEYS` with the keys:
//! `wait:ready`, `wait:answer`, `sleep:<ms>`, `goto:<line>:<col>` (1-based),
//! `hover:<line>:<col>` (a real mouse move), `click:<line>:<col>` (a real
//! click), `mark:<name>` (prints a line a capture script waits for, then
//! pauses), `state` (prints what the window shows). Keys and mouse go
//! through the window's own dispatch.

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
        ["click", line, col] => click(view, cx, (number(line), number(col))),
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
fn hover(view: &WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext, at: (usize, usize)) {
    let _gone: gpui::Result<()> = cx.update(|window: &mut Window, cx: &mut gpui::App| {
        if let Some(position) = pointer(view, window, cx, at, true) {
            window.dispatch_event(
                PlatformInput::MouseMove(MouseMoveEvent {
                    position,
                    pressed_button: None,
                    modifiers: gpui::Modifiers::default(),
                }),
                cx,
            );
        }
    });
}

/// A real left click on the boundary before char `col` on `line`: it
/// focuses the editor and puts the cursor there, as a person's click does.
fn click(view: &WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext, at: (usize, usize)) {
    let _gone: gpui::Result<()> = cx.update(|window: &mut Window, cx: &mut gpui::App| {
        let Some(position): Option<gpui::Point<gpui::Pixels>> =
            pointer(view, window, cx, at, false)
        else {
            return;
        };
        let (button, modifiers): (gpui::MouseButton, gpui::Modifiers) =
            (gpui::MouseButton::Left, gpui::Modifiers::default());
        window.dispatch_event(
            PlatformInput::MouseDown(gpui::MouseDownEvent {
                button,
                position,
                modifiers,
                click_count: 1,
                first_mouse: false,
            }),
            cx,
        );
        window.dispatch_event(
            PlatformInput::MouseUp(gpui::MouseUpEvent {
                button,
                position,
                modifiers,
                click_count: 1,
            }),
            cx,
        );
    });
}

/// Where char `col` of `line` is drawn: its middle, or the boundary before it.
fn pointer(
    view: &WeakEntity<ExplorerView>,
    window: &Window,
    cx: &mut gpui::App,
    (line, col): (usize, usize),
    middle: bool,
) -> Option<gpui::Point<gpui::Pixels>> {
    let view: gpui::Entity<ExplorerView> = view.upgrade()?;
    let (left, top, scrolled, text): (f32, f32, f32, String) =
        view.read_with(cx, |v: &ExplorerView, _: &gpui::App| {
            let state: std::cell::Ref<'_, gpui::UniformListScrollState> =
                v.editor_scroll.0.borrow();
            let bounds: gpui::Bounds<gpui::Pixels> = state.base_handle.bounds();
            (
                v.panel_width + 6.0 + crate::editor::GUTTER_W,
                f32::from(bounds.origin.y),
                f32::from(state.base_handle.offset().y),
                v.tabs
                    .active()
                    .and_then(|doc: &bb_text::Document| doc.line(line))
                    .unwrap_or_default(),
            )
        });
    let row: f32 = f32::from(u16::try_from(line).unwrap_or(u16::MAX));
    let columns: crate::editor::Columns =
        crate::editor::Columns::shape(window, text.trim_end_matches(['\r', '\n']));
    let x: f32 = if middle {
        f32::midpoint(columns.x_of(col), columns.x_of(col.saturating_add(1)))
    } else {
        columns.x_of(col)
    };
    Some(gpui::point(
        gpui::px(left + x),
        gpui::px(top + scrolled + 22.0 * row + 11.0),
    ))
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
