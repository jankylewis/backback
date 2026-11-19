# BB-UIX-008 — fix

id: BB-UIX-008
family: UIX
status: FIXED
source: branch `bb-bug-016-017-select-all-comment`

## The fix that holds

Two commands in the text model, `select_all` and `toggle_line_comment`
(after the indent, blank lines skipped, all-commented lines uncommented,
one undo step, the selection kept), and their arms in `command_key`.

## The wrong fix

Commenting at column 0 (VS Code and Rider comment after the indent), or
treating Cmd+/ as a plain char.

## Proof

Red first: the two unit tests in `lines_tests.rs` and the window test;
readback in `docs/plan/readback/V2/SWEEP01/bb_READBACK.md`.
