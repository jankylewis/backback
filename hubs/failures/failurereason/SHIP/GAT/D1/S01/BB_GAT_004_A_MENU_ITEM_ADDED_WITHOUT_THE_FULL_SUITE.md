# BB-GAT-004 — A menu item added without the full suite

id: BB-GAT-004
family: GAT
status: FIXED
date: 2026-11-08
signature: menus_go_app_then_file_then_view_as_on_every_mac ... assertion `left == right` failed: 8 vs 7
source: the word wrap branch's local gate (self-detected)
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

Word wrap added View > Word Wrap. Before the gate I ran only the wrap and
editor tests; the menu test that counts View's items (7) lives in
`app/bb_launch/tests.rs` and failed in the full gate. The product was
right; the count in the test was stale.
