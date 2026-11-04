# BB-BUG-040 — No split editor: a tab cannot be dragged to the right side

id: BB-BUG-040
family: UIX
status: FIXED
date: 2026-11-04
signature: one editor pane only; dragging a tab did nothing
source: headmate request 2026-11-04 ("the ability to drag a file splitted into the right side, like vs code")
cause: not built (a VS Code feature the editor lacked)
links: fix=yes lesson=no evidence=yes toolgap=none

## Steps

Open two files; drag one tab onto the right half of the editor.

Expected (VS Code): a second editor group opens on the right with that
file; both panes show text; a click in either focuses it.
Actual: nothing happened.
