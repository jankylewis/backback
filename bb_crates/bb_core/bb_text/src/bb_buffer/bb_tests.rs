//! Buffer tests: edits, bounds, UTF-16 mapping, LSP clamping, properties.

use super::*;
use proptest::prelude::*;

#[test]
fn insert_then_remove_restores_text() {
    let mut b: Buffer = Buffer::from_text("class A {}\n");
    b.insert(CharIdx::new(6), "B").unwrap();
    assert_eq!(b.to_text(), "class BA {}\n");
    b.remove(CharIdx::new(6), CharIdx::new(7)).unwrap();
    assert_eq!(b.to_text(), "class A {}\n");
}

#[test]
fn insert_out_of_bounds_is_an_error_not_a_panic() {
    let mut b: Buffer = Buffer::from_text("ab");
    assert_eq!(
        b.insert(CharIdx::new(3), "x"),
        Err(TextError::CharOutOfBounds { index: 3, len: 2 })
    );
}

#[test]
fn utf16_column_counts_surrogate_pairs_twice() {
    // "😀" is one char but two UTF-16 code units.
    let b: Buffer = Buffer::from_text("😀x\ny");
    let x: LspPoint = b.to_lsp(CharIdx::new(1)).unwrap();
    assert_eq!(
        x,
        LspPoint {
            line: 0,
            character: 2
        }
    );
    let y: LspPoint = b.to_lsp(CharIdx::new(3)).unwrap();
    assert_eq!(
        y,
        LspPoint {
            line: 1,
            character: 0
        }
    );
    assert_eq!(b.from_lsp(y).unwrap(), CharIdx::new(3));
}

#[test]
fn column_past_line_end_is_clamped_to_that_line() {
    let b: Buffer = Buffer::from_text("ab\r\ncd");
    let past: LspPoint = LspPoint {
        line: 0,
        character: 9,
    };
    assert_eq!(b.from_lsp(past).unwrap(), CharIdx::new(2));
    let missing: LspPoint = LspPoint {
        line: 7,
        character: 0,
    };
    assert!(matches!(
        b.from_lsp(missing),
        Err(TextError::LineOutOfBounds { .. })
    ));
}

proptest! {
    #[test]
    fn lsp_roundtrip_for_every_char(text in "[a-z😀é\n]{0,40}", pick in 0usize..41) {
        let b: Buffer = Buffer::from_text(&text);
        let idx: CharIdx = CharIdx::new(pick.min(b.len_chars()));
        let point: LspPoint = b.to_lsp(idx).unwrap();
        prop_assert_eq!(b.from_lsp(point).unwrap(), idx);
    }
}

proptest! {
    #[test]
    fn load_scan_and_edit_scan_agree_on_other_line_breaks(
        text in "[a\r\n\u{0b}\u{0c}\u{85}\u{2028}\u{2029}]{0,24}",
    ) {
        let buffer: Buffer = Buffer::from_text(&text);
        let whole: bool = buffer.other_break_near(CharIdx::new(0), CharIdx::new(buffer.len_chars()));
        prop_assert_eq!(Buffer::text_has_other_break(&text), whole);
    }
}
