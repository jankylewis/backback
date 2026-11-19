# BB-UIX-022 — C#-only chrome was not tied to the window's languages

id: BB-UIX-022
family: UIX
status: FIXED
date: 2026-11-19
signature: `C#` before a `.py`/`.rs` tab name; `C#`/`Debug` menus with no C#
source: `bb_view/mod.rs:172`, `bb_launch/mod.rs` `menus` at b13fcbb
observed: BB-BUG-085, BB-BUG-086
links: fix=yes lesson=yes evidence=no toolgap=none

## What it is

Two pieces of window chrome came from the C#-only app and never asked `Langs`: the tab bar drew a literal `C# ` mark, and the menu bar was set once at launch with Debug and C# menus. BBEOD26 gated Debug's keys and buttons by `Langs::debug_visible` but not its menu.

## How to confirm it

Open a Rust + Python folder; read the tab marks and the menu bar.
