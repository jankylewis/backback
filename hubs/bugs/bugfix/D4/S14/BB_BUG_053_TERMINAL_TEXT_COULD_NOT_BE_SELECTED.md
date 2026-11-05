# BB-BUG-053 — fix

id: BB-BUG-053
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/bb_explorer/panels/terminal/` (`bb_session.rs`: `select_from`, `select_to`, `selected_text`, `selected_rows`; `view/bb_mouse.rs`; `view/mod.rs`: `row`, Cmd+C)

## The change

The rows' box records its place each frame (a `ScrollHandle`); a press
starts alacritty's own selection at the cell under the mouse (left or
right half), a drag extends it, a release ends it. Because the selection
lives in alacritty's grid it moves with the text as output scrolls, and
`selection_to_string` gives the text with line breaks. Selected cells draw
in the theme's `selection` (BB-BUG-054); Cmd+C copies; a key typed into
the shell clears it. The view became `view/` (mouse, tests) and the
session tests moved to `tests.rs` (250-line cap).

## Lesson

A terminal selection belongs in the terminal model, not over the drawn
rows: alacritty already keeps it in grid points across scrolling. In gpui
tests the PTY never wakes the view, so a state change from a key needs its
own `cx.notify()` to be drawn.

## Proof

Session test: a selection over echoed output reads back exactly and
shows as a column range. Window test: a mouse drag selects (drawn), Cmd+C
puts exactly `bb_copy_me` on the clipboard, a key clears it
(mutation-checked: fails without the drag update).
