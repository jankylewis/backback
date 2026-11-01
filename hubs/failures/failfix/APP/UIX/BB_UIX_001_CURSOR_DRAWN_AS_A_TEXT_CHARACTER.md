# BB-UIX-001 — fix

id: BB-UIX-001
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/editor/view/bb_line/caret.rs` (`Columns`)
source: `docs/plan/readback/FIXES/BBEOD12/bb_READBACK.md`

## The fix that holds

One `Columns` per line, built from gpui's shaped line: `x_of(column)` and
`column_at(x)`. The row paints the same text as one `StyledText` (colour,
selection and squiggles are run styles); the caret is a quad at `x_of`.
`char_at`, the popup anchor and the script hover use `Columns`; the 7.8 px
constant is gone.

## The wrong fix

Inserting a zero-width char, or keeping a constant char width: tabs, emoji
and any non-Menlo fallback glyph break it.

## Proof

4 window tests red on master, green on the branch (emoji + tab line); gate
green; readback pixel diff in `docs/plan/readback/FIXES/BBEOD12/bb_READBACK.md`.
