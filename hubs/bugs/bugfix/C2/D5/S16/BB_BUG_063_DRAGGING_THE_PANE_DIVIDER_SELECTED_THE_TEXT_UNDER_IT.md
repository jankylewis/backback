# BB-BUG-063 — fix

id: BB-BUG-063
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/editor/view/bb_line/mod.rs` (`mouse_move`), `editor/input/link/clicks.rs` (`press`), `editor/view/pane/drag.rs` (`Lines::text_press`)

## The change

A press on the editor's text sets `Lines::text_press`; a release anywhere
(`finish_resize`) or a move with no button held clears it. A drag selects
only while it is set, so a drag that began on a divider, a scroll bar's
thumb, a tab or another panel never selects text. This was the only place
that extends a selection on a drag.

## Lesson

A drag handler must ask where the press began, not only that a button is
held: every draggable thing on screen sends the same moves over the text.

## Proof

`dragging_the_pane_divider_over_text_selects_nothing` (window test, long
lines as in the report) was red on master: a 37-char selection after the
divider drag. Green after; its control (a drag that starts on the text
still selects) passes. Making every press count as a text press turns it
red again. Gate green.
