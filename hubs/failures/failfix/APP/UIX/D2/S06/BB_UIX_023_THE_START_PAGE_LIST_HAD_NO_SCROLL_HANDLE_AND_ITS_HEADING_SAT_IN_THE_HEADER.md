# BB-UIX-023 — fix

id: BB-UIX-023
family: UIX
status: FIXED
source: branch `fix/readback-suspects`

## The fix that holds

RECENT is its own element placed right above the rows; the rows box tracks a `ScrollHandle` and `step` calls `scroll_to_item(selected)`.

## The wrong fix

A visible scroll bar alone: the keys would still leave the selection out of sight.
