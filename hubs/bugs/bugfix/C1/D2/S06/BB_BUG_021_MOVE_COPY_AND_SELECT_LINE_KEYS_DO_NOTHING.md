# BB-BUG-021 — fix

id: BB-BUG-021
family: UIX
status: FIXED
source: `bb_crates/bb_core/bb_text/src/document/movement/line_moves.rs` (`move_lines`, `copy_lines`), `bb_crates/shell/desktop/src/editor/input/bb_keys.rs` (`text_key`)

## The change

`move_lines(up)` swaps the lines the cursor or selection touches with the
line above or below; `copy_lines(up)` copies them above (the cursor stays)
or below (the cursor goes to the copy). One undo step each (`replace_many`).
A line keeps its own break (CRLF too); a last line without a break gets
one only where a line now follows it; the selection moves with its text.
Alt+Up/Down and Shift+Alt+Up/Down are refused in a read-only tab.

## Proof

Five unit tests (swap and back with one undo step each, the first and
last line stay, a last line without a break, CRLF, a selection over three
lines, copy above and below with and without a final break); window test
`alt_arrows_move_and_shift_alt_arrows_copy_the_line` (real keystrokes).
