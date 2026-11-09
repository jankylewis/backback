# BB-BUG-050 — No New File, New Folder, Rename or Delete in the Explorer

id: BB-BUG-050
family: UIX
status: FIXED
date: 2026-11-05
signature: a right-click on an Explorer row did nothing
source: headmate request 2026-11-05 (screenshot: "the ability to right click on a file -> show options: rename, delete, or create new file/folder")
cause: not built: the Explorer only opened and folded entries
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Right-click a file in the Explorer.

Expected: a menu as in VS Code: New File..., New Folder..., Rename..., Delete.
Actual: nothing.
