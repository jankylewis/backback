# BB-BUG-057 — fix

id: BB-BUG-057
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/bb_explorer/panels/bb_quick_open/bb_model/bb_line.rs`, `view.rs` (`key`, `query_line`)

## The change

`QuickOpen` now has a `caret` (a byte offset on a char boundary). Typing
and Cmd+V go in at the caret (a pasted text stays one line); Backspace and
Delete work around it; Left/Right move a char, Alt by word, Cmd or
Home/End to an end. The query line draws a caret bar between the text
before and after it. A rename box starts with the caret at the end.

## Lesson

A text box is more than a string: it needs a caret, paste and the arrow
keys from day one. Every box in the app uses this one model now.

## Proof

`the_find_box_pastes_at_a_caret_the_arrows_move` (window test): red before
the fix (query stayed `"ab"`, paste lost), green after. Unit tests cover
multi-byte chars, Delete at the end, line breaks in a paste and word jumps.
