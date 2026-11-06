//! The box's one-line text with a caret (BB-BUG-057): type and paste at
//! the caret, Backspace and Delete around it, and arrow keys move it, as
//! the VS Code input boxes do. The caret is a byte offset on a char
//! boundary of the text.

impl super::QuickOpen {
    /// One key press in the box; see [`press`].
    pub(crate) fn press(&mut self, stroke: &gpui::Keystroke, paste: Option<String>) {
        press(self, stroke, paste);
    }

    /// Type or paste `text` at the caret.
    pub(crate) fn type_text(&mut self, text: &str) {
        insert(&mut self.query, &mut self.caret, text);
        self.selected = 0;
    }

    pub(crate) fn backspace(&mut self) {
        backspace(&mut self.query, &mut self.caret);
        self.selected = 0;
    }

    /// Delete: the char after the caret.
    pub(crate) fn delete(&mut self) {
        delete(&mut self.query, self.caret);
        self.selected = 0;
    }

    /// Move the caret.
    pub(crate) fn caret_to(&mut self, to: Move) {
        self.caret = step(&self.query, self.caret, to);
    }

    /// The query split at the caret: (before, after).
    pub(crate) fn split(&self) -> (&str, &str) {
        self.query
            .split_at_checked(self.caret)
            .unwrap_or((&self.query, ""))
    }
}

/// A key the box edits with: arrows move the caret (Alt by word, Cmd or
/// Home/End to an end), Backspace/Delete remove, `paste` is Cmd+V's text.
pub(crate) fn press(quick: &mut super::QuickOpen, stroke: &gpui::Keystroke, paste: Option<String>) {
    let platform: bool = stroke.modifiers.platform;
    match (stroke.key.as_str(), platform, stroke.modifiers.alt) {
        ("backspace", _, _) => quick.backspace(),
        ("delete", _, _) => quick.delete(),
        ("up", _, _) => quick.step(false),
        ("down", _, _) => quick.step(true),
        ("left", true, _) | ("home", _, _) => quick.caret_to(Move::Start),
        ("right", true, _) | ("end", _, _) => quick.caret_to(Move::End),
        ("left", false, true) => quick.caret_to(Move::WordLeft),
        ("right", false, true) => quick.caret_to(Move::WordRight),
        ("left", false, false) => quick.caret_to(Move::Left),
        ("right", false, false) => quick.caret_to(Move::Right),
        (_, true, _) => {
            if let Some(text) = paste {
                quick.type_text(&text);
            }
        }
        (_, false, _) => {
            if let Some(text) = stroke.key_char.as_deref() {
                quick.type_text(text);
            }
        }
    }
}

/// Where an arrow key moves the caret.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Move {
    /// One char left (Left).
    Left,
    /// One char right (Right).
    Right,
    /// To the start of the word before (Alt+Left).
    WordLeft,
    /// To the end of the word after (Alt+Right).
    WordRight,
    /// To the start (Home, Cmd+Left).
    Start,
    /// To the end (End, Cmd+Right).
    End,
}

/// Put `typed` at the caret; a paste keeps one line: its line breaks at
/// the end go, and the ones inside become spaces.
pub(crate) fn insert(text: &mut String, caret: &mut usize, typed: &str) {
    let line: String = typed
        .trim_end_matches(['\r', '\n'])
        .replace("\r\n", " ")
        .replace(['\r', '\n'], " ");
    let at: usize = (*caret).min(text.len());
    if text.is_char_boundary(at) {
        text.insert_str(at, &line);
        *caret = at.saturating_add(line.len());
    }
}

/// Backspace: drop the char before the caret.
pub(crate) fn backspace(text: &mut String, caret: &mut usize) {
    let before: usize = step(text, *caret, Move::Left);
    text.replace_range(before..*caret, "");
    *caret = before;
}

/// Delete: drop the char after the caret.
pub(crate) fn delete(text: &mut String, caret: usize) {
    let after: usize = step(text, caret, Move::Right);
    text.replace_range(caret..after, "");
}

/// The caret after `to` from `caret`.
pub(crate) fn step(text: &str, caret: usize, to: Move) -> usize {
    let caret: usize = caret.min(text.len());
    let head: &str = text.get(..caret).unwrap_or(text);
    let tail: &str = text.get(caret..).unwrap_or("");
    match to {
        Move::Left => head
            .char_indices()
            .next_back()
            .map_or(0, |(at, _): (usize, char)| at),
        Move::Right => tail
            .chars()
            .next()
            .map_or(caret, |ch: char| caret.saturating_add(ch.len_utf8())),
        Move::WordLeft => {
            let trimmed: &str = head.trim_end_matches(|ch: char| !ch.is_alphanumeric());
            trimmed
                .rfind(|ch: char| !ch.is_alphanumeric())
                .map_or(0, |at: usize| {
                    let skipped: usize = trimmed
                        .get(at..)
                        .and_then(|rest: &str| rest.chars().next())
                        .map_or(1, char::len_utf8);
                    at.saturating_add(skipped)
                })
        }
        Move::WordRight => {
            let gap: usize = tail.len().saturating_sub(
                tail.trim_start_matches(|ch: char| !ch.is_alphanumeric())
                    .len(),
            );
            let rest: &str = tail.get(gap..).unwrap_or("");
            let word: usize = rest
                .find(|ch: char| !ch.is_alphanumeric())
                .unwrap_or(rest.len());
            caret.saturating_add(gap).saturating_add(word)
        }
        Move::Start => 0,
        Move::End => text.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::{backspace, delete, insert, step, Move};

    #[test]
    fn typing_goes_in_at_the_caret_and_the_caret_moves_by_chars() {
        let mut text: String = String::from("helo");
        let mut caret: usize = step(&text, 4, Move::Left);
        insert(&mut text, &mut caret, "l");
        assert_eq!((text.as_str(), caret), ("hello", 4));
        caret = step(&text, caret, Move::Start);
        insert(&mut text, &mut caret, "é");
        assert_eq!((text.as_str(), caret), ("éhello", 2));
        assert_eq!(step(&text, caret, Move::Left), 0, "one char, two bytes");
        backspace(&mut text, &mut caret);
        assert_eq!((text.as_str(), caret), ("hello", 0));
        backspace(&mut text, &mut caret);
        delete(&mut text, caret);
        assert_eq!(text, "ello");
        delete(&mut text, 4);
        assert_eq!(text, "ello", "Delete at the end does nothing");
    }

    #[test]
    fn a_pasted_text_stays_one_line() {
        let mut text: String = String::from("[]");
        let mut caret: usize = 1;
        insert(&mut text, &mut caret, "a\r\nb\nc\n");
        assert_eq!((text.as_str(), caret), ("[a b c]", 6));
    }

    #[test]
    fn alt_arrows_jump_words() {
        let text: &str = "var x = Needle();";
        assert_eq!(step(text, 17, Move::WordLeft), 8);
        assert_eq!(step(text, 8, Move::WordLeft), 4);
        assert_eq!(step(text, 0, Move::WordRight), 3);
        assert_eq!(step(text, 5, Move::WordRight), 14);
        assert_eq!(step(text, 17, Move::End), 17);
    }
}
