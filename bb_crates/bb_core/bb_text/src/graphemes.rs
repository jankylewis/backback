//! Grapheme boundaries (BB-BUG-033): the cursor steps over what a reader
//! sees as one character (`e` + a combining accent, an emoji sequence, a
//! flag), never inside it. `\r\n` is one grapheme too, so a step never
//! splits a line break (BB-BUG-010). Each step reads a small window around
//! the cursor: the cost does not grow with the line.

use ropey::Rope;
use unicode_segmentation::UnicodeSegmentation;

/// Chars read on each side of the cursor; a longer cluster is split.
const WINDOW: usize = 64;

/// The grapheme boundary before char `at` (0 at the text's start).
#[must_use]
pub(crate) fn before(rope: &Rope, at: usize) -> usize {
    let start: usize = at.saturating_sub(WINDOW);
    let Some(window): Option<String> = slice(rope, start, at) else {
        return at.saturating_sub(1);
    };
    window
        .grapheme_indices(true)
        .next_back()
        .map_or(start, |(byte, _): (usize, &str)| {
            start.saturating_add(
                window
                    .get(..byte)
                    .map_or(0, |head: &str| head.chars().count()),
            )
        })
}

/// The grapheme boundary after char `at` (the text's length at its end).
#[must_use]
pub(crate) fn after(rope: &Rope, at: usize) -> usize {
    let len: usize = rope.len_chars();
    let end: usize = at.saturating_add(WINDOW).min(len);
    let Some(window): Option<String> = slice(rope, at, end) else {
        return at.saturating_add(1).min(len);
    };
    window
        .graphemes(true)
        .next()
        .map_or(at, |first: &str| at.saturating_add(first.chars().count()))
}

fn slice(rope: &Rope, from: usize, to: usize) -> Option<String> {
    rope.get_slice(from..to)
        .map(|part: ropey::RopeSlice<'_>| part.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_cross_a_combining_accent_an_emoji_and_crlf_whole() {
        // a, e + U+0301, 👍 (one char), 👨‍👩‍👧 (five chars), \r\n, b
        let rope: Rope = Rope::from_str("ae\u{301}👍👨\u{200d}👩\u{200d}👧\r\nb");
        let mut at: usize = 0;
        let mut stops: Vec<usize> = vec![0];
        while at < rope.len_chars() {
            at = after(&rope, at);
            stops.push(at);
        }
        assert_eq!(stops, [0, 1, 3, 4, 9, 11, 12]);
        let mut back: Vec<usize> = vec![at];
        while at > 0 {
            at = before(&rope, at);
            back.push(at);
        }
        back.reverse();
        assert_eq!(back, stops, "the same stops both ways");
    }

    #[test]
    fn plain_text_steps_one_char() {
        let rope: Rope = Rope::from_str("int x;");
        assert_eq!(after(&rope, 0), 1);
        assert_eq!(before(&rope, 6), 5);
        assert_eq!(after(&rope, 6), 6, "the end stays");
        assert_eq!(before(&rope, 0), 0, "the start stays");
    }
}
