# BB-BUG-052 — No Copy, Paste or Open Terminal Here in the Explorer

id: BB-BUG-052
family: UIX
status: FIXED
date: 2026-11-05
signature: the Explorer's right-click menu had only New File, New Folder, Rename and Delete
source: headmate request 2026-11-05 ("more features: Open a Terminal here, Copy this file/folder")
cause: not built
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Right-click a file or folder in the Explorer.

Expected: Open Terminal Here, Copy, Paste (after a Copy) and Copy Path, as VS Code.
Actual: no such rows.
