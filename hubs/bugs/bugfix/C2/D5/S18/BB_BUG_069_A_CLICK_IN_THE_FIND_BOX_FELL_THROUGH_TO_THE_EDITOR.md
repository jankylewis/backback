# BB-BUG-069 — fix

id: BB-BUG-069
family: UIX
status: FIXED
source: `bb_quick_open/view/mod.rs`, `intel/view/bb_popups.rs`, the TESTS, Explorer and tab menus

## The change

The Find box, the completion / hover popups and the three right-click menus
occlude what is under them, so a press inside them reaches only them.

## Lesson

An overlay that only paints on top is not on top for the mouse. Every
`deferred` overlay occludes; a layout change that moves what is under an
overlay is what shows the gap.

## Proof

`the_scope_buttons_and_alt_keys_work_in_the_window`: red with the 32 px
title bar and no occlusion, green with it.
