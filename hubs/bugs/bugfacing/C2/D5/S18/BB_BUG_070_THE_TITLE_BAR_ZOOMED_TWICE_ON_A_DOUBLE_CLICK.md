# BB-BUG-070 — The title bar zoomed twice on a double click

id: BB-BUG-070
family: UIX
status: FIXED
date: 2026-11-18
signature: double-click the title bar: the window zooms, then flips back to its old size
source: headmate screen recording (2026-11-18), 12 s
cause: every press on the bar called `start_window_move` (macOS `performWindowDragWithEvent`), the second press of a double click too, so the system zoomed a second time after backback's own zoom
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Double-click the title bar.

Expected: The window zooms once.
Actual: It zooms and flips back (on, off).
