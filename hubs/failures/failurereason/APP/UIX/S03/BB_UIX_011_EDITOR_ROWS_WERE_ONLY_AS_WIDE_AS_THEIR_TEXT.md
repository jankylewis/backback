# BB-UIX-011 — Editor rows were only as wide as their text

id: BB-UIX-011
family: UIX
status: FIXED
date: 2026-11-03
signature: a click right of the text does nothing; the current line's band ends at the text
source: `bb_crates/shell/desktop/src/editor/view/bb_line/mod.rs` (`row_frame`)
observed: the headmate's report (BB-BUG-022)
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

Each editor row is a flex div that carries the mouse handlers and the
current-line background. It had no width, so it took the width of its
children: the gutter and the text. Since BB-BUG-013 the text sits in a
`flex_1` viewport, which grows only inside a row wider than its content. A
click right of the text hit no row, so no handler ran.

## How it shows

A click in the empty space right of a line does nothing; the current
line's band stops right after its text.

## How to confirm it

`a_click_past_the_line_end_puts_the_cursor_at_the_end`: a click 12 px
inside `editor-area`'s right edge.
