# BB-BUG-051 — fix

id: BB-BUG-051
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/editor/view/chrome/bb_scrollbar/` (`thumb`, `bars`; `bb_input.rs`: `drag`), `pane/drag.rs` (`Lines.bar`)

## The change

Each pane draws a bar on its right edge while its file is taller than the
pane; the focused pane also one at the bottom while a line in sight is
wider (the same widest line the sideways scroll stops at). The thumb's
length is the share in sight (at least 24 px), its place the scroll's;
dragging it scrolls from where the press began, a press on the track
centres the thumb there. No bar when everything fits. The status bar and
test marks moved with it to `editor/view/chrome/` (five-entry cap).

## Lesson

gpui's `uniform_list` keeps its scroll in `base_handle`: `bounds()` and
`max_offset()` of the last frame are enough to draw a bar, no second
measurement of the lines.

## Proof

`the_thumb_shows_the_share_in_sight_and_where` (unit), and window tests:
no bar for a short file; a 400-line file's thumb drags the lines and a
track press jumps; a 1,500-char line's bottom thumb scrolls across.
