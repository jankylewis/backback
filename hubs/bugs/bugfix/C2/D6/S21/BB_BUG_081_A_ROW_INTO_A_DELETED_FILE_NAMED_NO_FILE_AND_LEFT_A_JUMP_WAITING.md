# BB-BUG-081 — fix

id: BB-BUG-081
family: UIX
status: FIXED
source: branch `fix/nav-back-closed-file`; `editor::open_failed` (`editor/bb_tabs/model/history/mod.rs`), called from both open error arms in `main.rs`

## The change

A failed open drops the jump that waited for that file (`intel.jump`,
`tests.pending_jump`) and says `Cannot open <name>: <error>`.

## Proof

`a_places_row_reopens_its_closed_file_and_a_deleted_one_says_why_and_leaves_no_jump`
red before (`file path: No such file or directory (os error 2)`),
green after; `an_unreadable_file_says_why_and_leaves_no_waiting_jump_from_a_row_or_a_test`
and mutations M5 (intel jump kept), M6 (test jump kept), M7 (unnamed
error) all red.
