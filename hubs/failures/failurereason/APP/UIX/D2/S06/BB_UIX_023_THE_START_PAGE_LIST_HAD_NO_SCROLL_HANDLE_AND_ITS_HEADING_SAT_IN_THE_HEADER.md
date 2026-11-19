# BB-UIX-023 — The start page list had no scroll handle and its heading sat in the header

id: BB-UIX-023
family: UIX
status: FIXED
date: 2026-11-19
signature: RECENT above LANGUAGES; Down leaves the selection off screen
source: `bb_start/entries.rs` `header`, `bb_start/mod.rs` `step` at b13fcbb
observed: BB-BUG-087
links: fix=yes lesson=yes evidence=no toolgap=none

## What it is

`header` ended with RECENT, and BBEOD26 inserted the LANGUAGES row after the header, between RECENT and its rows. The rows box scrolled by wheel but had no `ScrollHandle`, so Up/Down changed `selected` without moving the view.

## How to confirm it

Start page with 10 recent folders in a 360 px window; press Down 10 times; read Open...'s bounds.
