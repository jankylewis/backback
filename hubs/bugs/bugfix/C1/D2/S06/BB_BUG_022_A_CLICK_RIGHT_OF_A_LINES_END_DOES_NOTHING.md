# BB-BUG-022 — fix

id: BB-BUG-022
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/editor/view/bb_line/mod.rs` (`row_frame`), `bb_crates/shell/desktop/src/app/script/bb_pointer.rs` (`click_past`)

## The change

Editor rows take the pane's width (`w_full`), so a click right of the text
lands on the row; `char_at` already maps an x past the text to the line's
end (`Columns::column_at`), and an empty line to its start. The readback
runner gains `clickpast:<line>`: a real click near the window's right edge.

## Proof

Red first: `a_click_past_the_line_end_puts_the_cursor_at_the_end` (window:
a click 12 px inside `editor-area`'s right edge left the cursor at 0).
Readback on the branch build, `clickpast`: line 9 (53 chars) -> col 54,
empty line 12 -> col 1, line 7 (38 chars) -> col 39.
