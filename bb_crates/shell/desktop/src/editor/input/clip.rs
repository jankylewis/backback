//! Cmd+C, Cmd+X and Cmd+V in the editor (BB-BUG-028), as in VS Code: with
//! no selection, copy and cut take the whole line, and a whole line pasted
//! with no selection goes in above the cursor's line.

use bb_text::{CharIdx, Document, DocumentError};
use gpui::{ClipboardItem, Context};

use crate::ExplorerView;

/// The clipboard key `key` (with Cmd), or `None` when it is not one.
pub(super) fn clipboard_key(
    view: &mut ExplorerView,
    key: &str,
    cx: &mut Context<ExplorerView>,
) -> Option<Result<(), DocumentError>> {
    match key {
        "c" | "x" => {
            let cut: bool = key == "x";
            let doc: &mut Document = view.tabs.active_mut()?;
            let (start, end, whole_line): (CharIdx, CharIdx, bool) = span(doc);
            let text: String = doc.rope().slice(start.raw()..end.raw()).to_string();
            if text.is_empty() {
                return Some(Ok(()));
            }
            let result: Result<(), DocumentError> = if cut {
                doc.select(start, end).and_then(|(): ()| doc.insert(""))
            } else {
                Ok(())
            };
            view.tabs.line_clip = whole_line.then(|| text.clone());
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            Some(result)
        }
        "v" => {
            let text: String = cx
                .read_from_clipboard()
                .and_then(|item: ClipboardItem| item.text())?;
            let as_line: bool = view.tabs.line_clip.as_deref() == Some(text.as_str());
            let doc: &mut Document = view.tabs.active_mut()?;
            Some(paste(doc, &text, as_line))
        }
        _ => None,
    }
}

/// The selection, else the cursor's whole line with its break; whether it
/// is a whole line.
fn span(doc: &Document) -> (CharIdx, CharIdx, bool) {
    if let Some((start, end)) = doc.selection() {
        return (start, end, false);
    }
    let line: usize = doc.cursor_line();
    let start: CharIdx = doc.line_start(line).unwrap_or(doc.cursor());
    let end: CharIdx = doc
        .line_start(line.saturating_add(1))
        .unwrap_or_else(|| CharIdx::new(doc.len_chars()));
    (start, end, true)
}

/// Paste `text` over the selection or at the cursor; a copied whole line
/// with no selection goes in above the cursor's line, the cursor staying
/// on its text.
fn paste(doc: &mut Document, text: &str, as_line: bool) -> Result<(), DocumentError> {
    if !as_line || doc.selection().is_some() {
        return doc.insert(text);
    }
    let cursor: CharIdx = doc.cursor();
    let start: CharIdx = doc.line_start(doc.cursor_line()).unwrap_or(cursor);
    doc.set_cursor(start)?;
    doc.insert(text)?;
    doc.set_cursor(CharIdx::new(
        cursor.raw().saturating_add(text.chars().count()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(text: &str, cursor: usize) -> Document {
        let mut doc: Document = Document::from_text(text, None);
        doc.set_cursor(CharIdx::new(cursor)).unwrap();
        doc
    }

    #[test]
    fn a_selection_spans_itself_and_no_selection_spans_the_line() {
        let mut d: Document = doc("ab\n\"4173-94\",\ncd", 4);
        assert_eq!(span(&d), (CharIdx::new(3), CharIdx::new(14), true));
        d.select(CharIdx::new(4), CharIdx::new(11)).unwrap();
        assert_eq!(span(&d), (CharIdx::new(4), CharIdx::new(11), false));
        let last: Document = doc("ab\ncd", 4);
        assert_eq!(
            span(&last),
            (CharIdx::new(3), CharIdx::new(5), true),
            "the last line has no break"
        );
    }

    #[test]
    fn a_copied_line_pastes_above_and_text_pastes_at_the_cursor() {
        let mut d: Document = doc("one\ntwo\n", 5);
        paste(&mut d, "one\n", true).unwrap();
        assert_eq!(d.text(), "one\none\ntwo\n");
        assert_eq!(d.cursor(), CharIdx::new(9), "still after the t of two");
        let mut e: Document = doc("one\ntwo\n", 5);
        paste(&mut e, "X", false).unwrap();
        assert_eq!(e.text(), "one\ntXwo\n");
    }

    #[test]
    fn a_line_paste_over_a_selection_replaces_it() {
        let mut d: Document = doc("one\ntwo\n", 0);
        d.select(CharIdx::new(4), CharIdx::new(7)).unwrap();
        paste(&mut d, "one\n", true).unwrap();
        assert_eq!(d.text(), "one\none\n\n");
    }
}
