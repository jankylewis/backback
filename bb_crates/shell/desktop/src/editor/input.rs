use gpui::{Context, KeyDownEvent, Window};
use std::time::Instant;

use crate::ExplorerView;

pub(crate) fn pointer_at(
    view: &mut ExplorerView,
    line: usize,
    window_x: f32,
    extend: bool,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let left: f32 = view.panel_width + 56.0;
    if let Some(document) = view.tabs.active_mut() {
        let start: usize = document.line_start(line).map_or(0, bb_text::CharIdx::raw);
        let content: String = document.line(line).unwrap_or_default();
        let visible: usize = content.trim_end_matches(['\r', '\n']).chars().count();
        let column: usize = (0..visible)
            .take_while(|index: &usize| {
                let step: u16 = u16::try_from(index.saturating_add(1)).unwrap_or(u16::MAX);
                7.8 * f32::from(step) <= (window_x - left).max(0.0)
            })
            .count();
        let target: bb_text::CharIdx =
            bb_text::CharIdx::new(start.saturating_add(column.min(visible)));
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
    if let Some(tab) = view.tabs.active_tab_mut() {
        tab.sync_edits();
        let path: std::path::PathBuf = tab.path.clone();
        schedule_parse(view, path, window, cx);
    }
    if timed {
        view.reparse_ns = view
            .reparse_ns
            .saturating_add(crate::latency::nanos(reparse_start.elapsed()));
        view.record_keypress(window);
    }
    cx.stop_propagation();
    cx.notify();
}

fn command_key(
    view: &mut ExplorerView,
    key: &str,
    shift: bool,
) -> Option<Result<(), bb_text::DocumentError>> {
    let result: Result<(), bb_text::DocumentError> = match key {
        "s" => view
            .tabs
            .active_mut()
            .map_or(Ok(()), bb_text::Document::save),
        "w" => {
            view.tabs.close_active();
            Ok(())
        }
        "z" if shift => view
            .tabs
            .active_mut()
            .map_or(Ok(()), bb_text::Document::redo),
        "z" => view
            .tabs
            .active_mut()
            .map_or(Ok(()), bb_text::Document::undo),
        "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
            if let Ok(number) = key.parse::<usize>() {
                view.tabs.switch(number.saturating_sub(1));
            }
            Ok(())
        }
        _ => return None,
    };
    Some(result)
}

fn text_key(
    document: &mut bb_text::Document,
    event: &KeyDownEvent,
    shift: bool,
) -> Option<Result<(), bb_text::DocumentError>> {
    let result: Result<(), bb_text::DocumentError> = match event.keystroke.key.as_str() {
        "backspace" => document.backspace(),
        "delete" => document.delete(),
        "enter" | "return" => document.enter(),
        "left" => document.left(shift),
        "right" => document.right(shift),
        "up" => document.vertical(false, shift),
        "down" => document.vertical(true, shift),
        _ => document.insert(event.keystroke.key_char.as_deref()?),
    };
    Some(result)
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
                    let outcome: Option<Result<(), bb_syntax::SyntaxError>> = view
                        .tabs
                        .tab_mut(&path)
                        .map(|tab: &mut super::Tab| tab.live.finish_parse(parsed));
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
