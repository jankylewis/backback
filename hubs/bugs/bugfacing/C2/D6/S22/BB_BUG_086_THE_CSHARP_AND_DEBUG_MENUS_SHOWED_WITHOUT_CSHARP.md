# BB-BUG-086 — The C# and Debug menus showed without C#

id: BB-BUG-086
family: UIX
status: FIXED
date: 2026-11-19
signature: menu bar `... Build  Debug  C#  Window` in a Rust + Python window
source: BBEOD32 readback rows 3, 4; `app/bb_launch/mod.rs` `menus` at b13fcbb
cause: BB-UIX-022
links: fix=yes lesson=yes evidence=no toolgap=none

## Steps

Open `py_rust` (no C#); look at the menu bar. Open `cs_only` too, then click back into the `py_rust` window.

Expected: No `C#` or `Debug` menu while the front window has no C# active (`Langs::debug_visible` already said so for keys and buttons).

Actual: Both menus show in every window: the bar was set once at launch.
