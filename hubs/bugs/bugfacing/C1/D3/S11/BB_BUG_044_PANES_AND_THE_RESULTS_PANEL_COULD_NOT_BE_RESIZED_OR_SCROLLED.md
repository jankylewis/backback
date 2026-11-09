# BB-BUG-044 — The panes and the results panel could not be resized; long results were cut off

id: BB-BUG-044
family: UIX
status: FIXED
date: 2026-11-04
signature: fixed 50/50 panes; a fixed 180 px results panel; result rows cut at the panel's edge
source: headmate requests 2026-11-04 (drag the line between the panes; drag the line above the Find in Files results; see the rightmost part of a result; the match before the path)
cause: not built: fixed sizes, rows clipped with an ellipsis, path first
links: fix=yes lesson=no evidence=yes toolgap=none

## Steps

Split the editor and drag the line between the panes; run Find in Files
(Cmd+Shift+F) and drag the line above the results; look for the end of a
long result.

Expected (VS Code): both lines drag; the results scroll sideways; the
match text leads.
Actual: nothing dragged; long rows ended in "…"; the path came first.
