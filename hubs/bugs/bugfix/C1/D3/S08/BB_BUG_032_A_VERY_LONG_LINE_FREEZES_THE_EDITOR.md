# BB-BUG-032 — fix

id: BB-BUG-032
family: PRF
status: FIXED
source: `bb_crates/shell/desktop/src/editor/view/bb_line/caret.rs` (`RENDER_CAP`, `rendered`), `bb_crates/shell/desktop/src/editor/view/bb_line/bb_runs.rs` (`styled_line`)

## The change

A line is drawn and measured up to 10 000 chars (VS Code's
`editor.stopRenderingLineAfter` default), then ends in `…`. The document
keeps every char: the cursor, edits and saves reach all of it.

## Proof

Unit test `a_long_line_is_cut_at_the_render_cap_and_a_short_one_is_kept`.
Readback on the branch build, same file: first state 11.3 -> 2.8 s (with
a 1.5 s sleep); goto + Right 42.3 -> 2.1 s; goto + Option+Right 42.1 ->
2.1 s; typing at column 5000 works (col 5001).
