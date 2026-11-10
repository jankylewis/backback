# BB-BUG-062 — fix

id: BB-BUG-062
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/editor/bb_tabs/bar/bb_view/mod.rs` (`tab_bar`, `Bars`)

## The change

The bar is a sideways scroll area (`overflow_x_scroll`) with one
`ScrollHandle` per group in `Tabs::bars`. A vertical wheel scrolls it too
(gpui maps it to x when y cannot scroll). When a group's active tab
changes, the bar scrolls it into sight once (`ScrollHandle::scroll_to_item`,
minimal move), so the wheel can still move away from it afterwards.

## Lesson

A row of items with `overflow_hidden()` loses whatever does not fit. Any
list that can grow (tabs, chips, breadcrumbs) needs a scroll, plus a
reveal of the item the user just acted on. Reveal on change, not on every
render, or it fights the user's own scrolling.

## Proof

`the_active_tab_scrolls_into_sight_and_the_wheel_brings_the_first_back`
(window test, 13 tabs) was red on master: the active last tab was out of
sight. Green after the change. Mutations: no reveal -> red; `overflow_hidden`
back -> red; reveal on every render -> red (the wheel cannot bring the first
tab back). Gate green.
