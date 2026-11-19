# BB-UIX-011 — fix

id: BB-UIX-011
family: UIX
status: FIXED
source: branch `bb-bug-022-click-past-line-end`

## The fix that holds

`w_full` on the row: the row is the element that owns the click and the
band, so it spans the pane, and the `flex_1` text viewport fills the rest.

## The wrong fix

A second hit test for clicks outside every row (a gap handler on the list):
two paths for one click, and the band would still end at the text.

## Proof

The red window test above; readback with `clickpast` (BB-BUG-022's page).
