use gpui::{Context, KeyDownEvent, Window};
use std::time::Instant;

use crate::ExplorerView;
mod bb_keys;
use bb_keys::{command_key, text_key};

/// The char under window position `window_x` on `line` of the active tab.
/// Measured on the same shaped line the row paints (BB-UIX-001).
pub(crate) fn char_at(
    view: &ExplorerView,
    line: usize,
    window_x: f32,
    window: &Window,
) -> Option<bb_text::CharIdx> {
    let left: f32 = view.panel_width + 6.0 + crate::editor::GUTTER_W;
    let document: &bb_text::Document = view.tabs.active()?;
    let start: usize = document.line_start(line)?.raw();
    let content: String = document.line(line).unwrap_or_default();
    let visible: &str = content.trim_end_matches(['\r', '\n']);
    let column: usize = crate::editor::Columns::shape(window, visible).column_at(window_x - left);
    Some(bb_text::CharIdx::new(start.saturating_add(column)))
}

pub(crate) fn pointer_at(
    view: &mut ExplorerView,
    line: usize,
    window_x: f32,
    extend: bool,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let target: Option<bb_text::CharIdx> = char_at(view, line, window_x, window);
    view.intel.dismiss();
    if let (Some(document), Some(target)) = (view.tabs.active_mut(), target) {
        if let Err(error) = document.move_to(target, extend) {
            view.status = Some(error.to_string());
        }
    }
    window.focus(&view.editor_focus, cx);
    cx.notify();
}

pub(crate) fn handle_key(
    view: &mut ExplorerView,
    event: &KeyDownEvent,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    if intercepted(view, event) {
        cx.stop_propagation();
        cx.notify();
        return;
    }
    let key: &str = event.keystroke.key.as_str();
    let shift: bool = event.keystroke.modifiers.shift;
    let timed: bool = event.keystroke.key_char.is_some() && !event.keystroke.modifiers.platform;
    let saved: bool = event.keystroke.modifiers.platform && key == "s";
    let edit_start: Instant = Instant::now();
    let result: Option<Result<(), bb_text::DocumentError>> = if event.keystroke.modifiers.platform {
        command_key(view, key, shift)
    } else {
        view.tabs
            .active_mut()
            .and_then(|document: &mut bb_text::Document| text_key(document, event, shift))
    };
    let Some(result): Option<Result<(), bb_text::DocumentError>> = result else {
        return;
    };
    if timed {
        view.edit_ns = view
            .edit_ns
            .saturating_add(crate::latency::nanos(edit_start.elapsed()));
    }
    match result {
        Ok(()) if saved => view.status = None,
        Ok(()) => {}
        Err(error) => view.report(&error),
    }
    let reparse_start: Instant = Instant::now();
    after_key(view, event, window, cx);
    if timed {
        view.reparse_ns = view
            .reparse_ns
            .saturating_add(crate::latency::nanos(reparse_start.elapsed()));
        view.record_keypress(window);
    }
    cx.stop_propagation();
    cx.notify();
}

/// Move the edits onto the tree and start a parse; keep the completion list
/// in step with what was typed.
fn after_key(
    view: &mut ExplorerView,
    event: &KeyDownEvent,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    if let Some(tab) = view.tabs.active_tab_mut() {
        tab.sync_edits();
        let path: std::path::PathBuf = tab.path.clone();
        schedule_parse(view, path, window, cx);
    }
    if !event.keystroke.modifiers.platform {
        crate::intel::glue::bb_keys::after_edit(view, event.keystroke.key_char.as_deref());
    }
}

/// Intel popups and commands take the key first; then a refused edit or
/// save stops it with a status line.
fn intercepted(view: &mut ExplorerView, event: &KeyDownEvent) -> bool {
    if crate::intel::glue::bb_keys::key(view, event) {
        return true;
    }
    let Some(why): Option<String> = refused(view, event) else {
        return false;
    };
    view.report(&why);
    true
}

/// Why the key must not run here, if it changes text in a read-only tab or
/// saves in no-write mode.
fn refused(view: &ExplorerView, event: &KeyDownEvent) -> Option<String> {
    let stroke: &gpui::Keystroke = &event.keystroke;
    let key: &str = stroke.key.as_str();
    let platform: bool = stroke.modifiers.platform;
    let saves: bool = platform && key == "s";
    if saves && view.tabs.no_write() {
        return Some(String::from("no-write mode: save refused"));
    }
    let edits: bool = if platform {
        key == "s" || key == "z"
    } else {
        matches!(key, "backspace" | "delete" | "enter" | "return") || stroke.key_char.is_some()
    };
    let read_only: bool = view
        .tabs
        .active_tab()
        .is_some_and(|tab: &super::Tab| tab.read_only);
    (edits && read_only).then(|| String::from("read-only file: edit refused"))
}

/// Start a background parse for the tab at `path` when edits wait and none
/// runs; on completion install it and start the next one if typing went on.
pub(super) fn schedule_parse(
    view: &mut ExplorerView,
    path: std::path::PathBuf,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let Some(tab): Option<&mut super::Tab> = view.tabs.tab_mut(&path) else {
        return;
    };
    let scope: Option<(bb_text::BytePoint, bb_text::BytePoint)> = tab.parse_window();
    let Some(job): Option<bb_syntax::ParseJob> = tab.live.start_parse(tab.document.rope(), scope)
    else {
        return;
    };
    let parse: gpui::Task<(bb_syntax::Parsed, std::time::Duration)> =
        cx.background_executor().spawn(async move {
            let started: Instant = Instant::now();
            let parsed: bb_syntax::Parsed = job.run();
            (parsed, started.elapsed())
        });
    cx.spawn_in(
        window,
        async move |view: gpui::WeakEntity<ExplorerView>, cx: &mut gpui::AsyncWindowContext| {
            let (parsed, took): (bb_syntax::Parsed, std::time::Duration) = parse.await;
            crate::latency::record_parse(took);
            view.update_in(
                cx,
                |view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>| {
                    let outcome: Option<Result<(), bb_syntax::SyntaxError>> =
                        view.tabs.tab_mut(&path).map(|tab: &mut super::Tab| {
                            let outcome: Result<(), bb_syntax::SyntaxError> =
                                tab.live.finish_parse(parsed);
                            if outcome.is_ok() {
                                tab.refresh_test_methods();
                            }
                            outcome
                        });
                    if let Some(Err(error)) = outcome {
                        view.report(&error);
                    }
                    schedule_parse(view, path, window, cx);
                    cx.notify();
                },
            )
            .ok();
        },
    )
    .detach();
}
