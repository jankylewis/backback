# BB-BUG-045 — Only two editor panes; three files could not show side by side

id: BB-BUG-045
family: UIX
status: FIXED
date: 2026-11-04
signature: a third Move to Right Group did nothing; one split line only
source: headmate request 2026-11-04 ("can it be 3? ... split into 3 file viewers at once")
cause: not built: the groups were Left and Right (BB-BUG-040)
links: fix=yes lesson=no evidence=yes toolgap=none

## Steps

Split the editor, then move a tab right of the right pane.

Expected (VS Code): a third pane.
Actual: it stayed in two.
