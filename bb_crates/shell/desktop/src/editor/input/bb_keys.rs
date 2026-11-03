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
        _ => {
            let command: DocumentCommand = document_command(key)?;
            view.tabs
                .active_mut()
                .map_or(Ok(()), |doc: &mut bb_text::Document| command(doc, shift))
        }
    };
    Some(result)
}

/// A Cmd key that acts on the document; `shift` as held.
type DocumentCommand = fn(&mut bb_text::Document, bool) -> Result<(), bb_text::DocumentError>;

/// Undo and redo, Select All (BB-BUG-016), the line comment (BB-BUG-017),
/// the deletes to a line's edges (BB-BUG-014), and the line commands of
/// sweep 4 (BB-BUG-020, BB-BUG-021): Cmd+Shift+K, Cmd+(Shift+)Enter, Cmd+L. Cmd+[ and Cmd+]
/// stay Back and Forward (app bindings, `open.rs`).
fn document_command(key: &str) -> Option<DocumentCommand> {
    Some(match key {
        "z" => |doc: &mut bb_text::Document, shift: bool| {
            if shift {
                doc.redo()
            } else {
                doc.undo()
            }
        },
        "a" => |doc: &mut bb_text::Document, _: bool| doc.select_all(),
        "/" => |doc: &mut bb_text::Document, _: bool| doc.toggle_line_comment(),
        "backspace" => |doc: &mut bb_text::Document, _: bool| doc.delete_to_line_start(),
        "delete" => |doc: &mut bb_text::Document, _: bool| doc.delete_to_line_end(),
        "l" => |doc: &mut bb_text::Document, _: bool| doc.select_line(),
        "k" => |doc: &mut bb_text::Document, shift: bool| {
            if shift {
                doc.delete_lines()
            } else {
                Ok(())
            }
        },
        "enter" => |doc: &mut bb_text::Document, shift: bool| {
            if shift {
                doc.insert_line_above()
            } else {
                doc.insert_line_below()
            }
        },
        _ => return None,
    })
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
    let option: bool = event.keystroke.modifiers.alt;
    if event.keystroke.modifiers.control {
        return control_key(document, event.keystroke.key.as_str(), shift);
    }
    let result: Result<(), bb_text::DocumentError> = match event.keystroke.key.as_str() {
        // Option: word moves and deletes, as in every macOS editor (BB-BUG-011).
        "backspace" if option => document.delete_word_back(),
        "delete" if option => document.delete_word_forward(),
        "left" if option => document.word_left(shift),
        "right" if option => document.word_right(shift),
        "backspace" => document.backspace(),
        "delete" => document.delete(),
        "enter" | "return" => document.enter(),
        "left" => document.left(shift),
        "right" => document.right(shift),
        "home" => document.home(shift),
        "end" => document.end(shift),
        "up" => document.vertical(false, shift),
        "down" => document.vertical(true, shift),
        // Tab indents, Shift+Tab outdents; a selection is never typed over
        // (BB-BUG-015).
        "tab" if shift => document.outdent(),
        "tab" => document.indent(),
        _ => document.insert(event.keystroke.key_char.as_deref()?),
    };
    Some(result)
}

/// The macOS text-system Ctrl keys (Cocoa, VS Code on macOS); any other
/// Ctrl key is not an edit and never inserts its letter.
fn control_key(
    document: &mut bb_text::Document,
    key: &str,
    shift: bool,
) -> Option<Result<(), bb_text::DocumentError>> {
    let result: Result<(), bb_text::DocumentError> = match key {
        "a" => document.line_start_move(shift),
        "e" => document.end(shift),
        "b" => document.left(shift),
        "f" => document.right(shift),
        "p" => document.vertical(false, shift),
        "n" => document.vertical(true, shift),
        "h" => document.backspace(),
        "d" => document.delete(),
        _ => return None,
    };
    Some(result)
}
