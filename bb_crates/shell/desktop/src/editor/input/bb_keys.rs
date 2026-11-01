//! Editor keys: Cmd shortcuts and the plain text keys.

use gpui::KeyDownEvent;

use crate::ExplorerView;

pub(super) fn command_key(
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
        "left" | "right" | "up" | "down" => view
            .tabs
            .active_mut()
            .map_or(Ok(()), |doc: &mut bb_text::Document| {
                command_move(doc, key, shift)
            }),
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

/// Cmd+Left/Right: line start (smart) and end; Cmd+Up/Down: document edges.
fn command_move(
    doc: &mut bb_text::Document,
    key: &str,
    shift: bool,
) -> Result<(), bb_text::DocumentError> {
    match key {
        "left" => doc.home(shift),
        "right" => doc.end(shift),
        "up" => doc.document_edge(false, shift),
        _ => doc.document_edge(true, shift),
    }
}

pub(super) fn text_key(
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
        "home" => document.home(shift),
        "end" => document.end(shift),
        "up" => document.vertical(false, shift),
        "down" => document.vertical(true, shift),
        _ => document.insert(event.keystroke.key_char.as_deref()?),
    };
    Some(result)
}
