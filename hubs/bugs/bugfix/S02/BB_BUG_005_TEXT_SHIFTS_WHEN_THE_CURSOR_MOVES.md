# BB-BUG-005 — fix

id: BB-BUG-005
family: UIX
status: FIXED
source: branch `bbeod-12-cursor` (BBEOD12 PR)
source: `docs/plan/readback/FIXES/S02/BBEOD12/bb_READBACK.md`

## The change

The cursor is a 2 px quad painted over the line; the line is one shaped text
whose glyphs are where `Columns` measures them. Clicks, popups and the script
hover read the same `Columns`. Cause: BB-UIX-001.

## Proof

Red first on master, green after: `the_line_text_is_the_file_line_at_every_cursor_column`,
`the_caret_sits_at_the_shaped_x_of_its_column_and_the_text_never_moves`,
`a_click_lands_on_the_column_drawn_under_the_mouse`,
`a_cursor_popup_is_anchored_at_the_caret_x`. Real-app readback: only the old
and new caret stripes differ between captures; the text pixels are identical.
