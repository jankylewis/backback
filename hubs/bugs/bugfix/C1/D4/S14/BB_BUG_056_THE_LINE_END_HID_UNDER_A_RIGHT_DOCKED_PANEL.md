# BB-BUG-056 — fix

id: BB-BUG-056
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/editor/view/pane/bb_hscroll.rs` (`text_viewport_width`), `bb_explorer/panels/terminal/view/mod.rs` (`grid`)

## The change

Both widths now come from the one `pane::area` (exported as
`editor::editor_area`), which already leaves a right-docked tool window and
its edge out. The horizontal scroll limit, the caret-in-sight rule and the
terminal's column count follow the real text width on either side.

## Lesson

Three copies of "how wide is the editor" drifted: only the pane layout knew
about the right dock. One width function, used everywhere; a test runs each
layout with the panel on both sides.

## Proof

`with_the_panel_on_the_right_the_line_end_scrolls_into_sight` (caret was at
x 1896 past the area's right edge 1614) and
`the_columns_are_the_same_with_the_panel_left_or_right` (204 vs 244): both
red before the fix, green after.
