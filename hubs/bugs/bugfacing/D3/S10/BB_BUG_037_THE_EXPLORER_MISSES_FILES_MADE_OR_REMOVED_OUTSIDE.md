# BB-BUG-037 — The Explorer misses files made or removed outside

id: BB-BUG-037
family: UIX
status: FIXED
date: 2026-11-04
signature: a file another tool made is not in the tree; a deleted one stays
source: night bug loop 2026-11-04, readback on the branch build
cause: BB-UIX-016
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Open a folder, expand a subfolder; from outside create a file and delete
another (git checkout, a generator, a terminal command).

Expected (VS Code): the tree shows the new file and drops the gone one.
Actual: the tree kept the rows it read: `a.py` stayed, `new.py` never
showed, and opening `new.py` could not select it in the tree.
