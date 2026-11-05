# BB-BUG-055 — fix

id: BB-BUG-055
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/editor/input/mod.rs` (`after_key`)

## The change

After a key that moved the cursor or changed the text (file, line count,
cursor line or cursor differ from before the key), the focused pane scrolls
to the cursor's line with `ScrollStrategy::Nearest`: just enough to show it,
and not at all when it is already in view. Save and other keys that move
nothing do not scroll, so a wheel-scrolled view stays where it is.

## Lesson

A cursor that can leave the screen needs one rule at the single key exit
(`after_key`), not per command: every movement key, edit and Cmd jump goes
through it.

## Proof

`the_lines_follow_the_cursor_up_and_down` (window test: Down x120 scrolls,
Up x120 back to 0, a move inside the view does not scroll). Red without the
fix (no scroll at line 120), green with it.
