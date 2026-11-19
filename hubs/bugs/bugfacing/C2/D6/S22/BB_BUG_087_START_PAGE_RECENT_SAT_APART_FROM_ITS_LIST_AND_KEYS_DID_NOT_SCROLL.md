# BB-BUG-087 — Start Page RECENT sat apart from its list and keys did not scroll

id: BB-BUG-087
family: UIX
status: FIXED
date: 2026-11-19
signature: RECENT above LANGUAGES; the last row cut at the bottom
source: BBEOD32 readback row 1a; `bb_start/entries.rs` `header`, `bb_start/mod.rs` `step` at b13fcbb
cause: BB-UIX-023
links: fix=yes lesson=yes evidence=no toolgap=none

## Steps

Launch with no argument and 8+ recent workspaces in a short window; press Down to the last row.

Expected: RECENT directly above its rows; Down/Up keep the selected row in sight; the wheel reaches Open....

Actual: RECENT sat in the page header, LANGUAGES between it and its rows; Down moved the selection off screen (no scroll handle), so the last row stayed cut.
