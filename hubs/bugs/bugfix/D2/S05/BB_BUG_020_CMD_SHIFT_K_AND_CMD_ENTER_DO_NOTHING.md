# BB-BUG-020 — fix

id: BB-BUG-020
family: UIX
status: FIXED
source: `bb_crates/bb_core/bb_text/src/document/edit/` (`indent_lines`, `delete_lines`, `insert_line_below`, `insert_line_above`), `bb_crates/shell/desktop/src/editor/input/bb_keys.rs` (`document_command`)

## The change

Whole-line commands in the text model, each one undo step: Cmd+Shift+K
deletes the touched lines with their break (the last line takes the break
before it), Cmd+Enter and Cmd+Shift+Enter open a line below or above with
the line's indent and the file's own line break.

## Proof

Red first: `cmd_shift_k_deletes_the_touched_lines_with_their_break` (CRLF,
a selection, and a last line without a newline, which caught a first
version that kept the break) and
`cmd_enter_and_cmd_shift_enter_open_a_line_below_and_above_with_its_indent`
(CRLF); window test `the_line_command_keys_reach_the_line_commands`.
Readback: sweep 4 K4, K6 and K7 PASS on the branch build.
