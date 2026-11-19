# BB-UIX-006 — fix

id: BB-UIX-006
family: UIX
status: FIXED
source: branch `bb-bug-013-hscroll`

## The fix that holds

One number per tab, `scroll_x`: the row's text viewport (what is left
after the gutter) is clipped and its shaped line sits that far to the
left; the caret and the cursor popups subtract it, clicks and hovers add
it. The wheel handler on the editor area takes the horizontal part of a
scroll and clamps it to the widest line in sight plus slack; `before_rows`
keeps the caret in sight only when the cursor moved, so a swipe stays.

## The wrong fix

Word wrap by default (hides the problem; VS Code and Rider default to no
wrap for code); a scroll per row (rows would disagree); a constant char
width for the scroll range (BB-UIX-001).

## Proof

`the_caret_stays_in_sight_on_a_long_line_and_a_swipe_scrolls` (window) and
the scroll-rule unit test; the readback in
`docs/plan/readback/V2/SWEEP01/bb_READBACK.md`.
