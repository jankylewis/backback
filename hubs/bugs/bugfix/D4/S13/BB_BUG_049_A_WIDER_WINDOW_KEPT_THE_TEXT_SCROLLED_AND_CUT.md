# BB-BUG-049 — fix

id: BB-BUG-049
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/editor/view/pane/bb_hscroll.rs` (`before_rows`)

## The change

Every frame, before the rows paint, the scroll is clamped to the widest
line in sight less the viewport, so a wider window (or a shorter line in
sight) pulls the text back.

## Lesson

A clamp that depends on the viewport belongs where the viewport is known
each frame, not only in the input handler that moves the scroll. But a
`uniform_list` first renders row 0 alone to measure the row height: a clamp
there sees only line 0 and threw a swipe back to 0 (the gate caught it in
`a_swipe_scrolls_within_the_text`), so the clamp skips that one-row pass.

## Proof

`a_wider_window_pulls_a_scrolled_line_back` (pane window tests): End on a
600-char line scrolls; a resize to 20,000 px brings the scroll to 0.
Without the clamp it stays at 3194 px (mutation checked).
