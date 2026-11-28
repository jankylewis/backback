# BB-BUG-095 — A long Find title hid the close x and stopped sideways scroll

id: BB-BUG-095
family: UIX
status: OBSERVED
date: 2026-11-26
signature: `N result for <query> in Project · <project> · <path>` runs past the panel's right edge; no `✕` in the title row
source: headmate report 2026-11-26 (BBEOD45 brief, shots `BBEOD45_FIND_PANEL_CANNOT_CLOSE` and `..._CANNOT_SCROLL_RIGHT`)
observed: Find in Project on a long test name; the results panel showed no close control, its long row could not be scrolled right, and the title and row text sat offset
cause: not proven as a family entry; the window test shows the title div's content width pushed the `✕` to x=4035 in a 1614 px panel
links: fix=yes lesson=no evidence=yes toolgap=none

## Steps

Find in Files (Cmd+Shift+F) for a long name, narrowed to a project, so the
title reads `1 result for <name> in Project · <project> · <path>, 1 file`.

Expected: the `✕` at the title row's right, the title cut with an ellipsis,
the rows aligned under it and scrolling sideways (BB-BUG-044).

Actual: the title kept its full width, so the `✕` was drawn outside the panel.
