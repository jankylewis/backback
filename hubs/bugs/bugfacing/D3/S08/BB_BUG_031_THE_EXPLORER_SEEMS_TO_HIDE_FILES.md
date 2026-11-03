# BB-BUG-031 — The Explorer seems to hide files

id: BB-BUG-031
family: UIX
status: OBSERVED
date: 2026-11-04
signature: a Python package folder in the Explorer shows only subfolders
source: the headmate's report 2026-11-04 ("critical": `MbAx/agentmates/autogen` shows `__pycache__`, `tests`, `tools`, no `.py`)
cause: BB-UIX-014
links: fix=no lesson=yes evidence=yes toolgap=none

## Steps

Open a folder whose Python packages hold only `__pycache__`.

Expected: every file and folder on disk is listed; any file opens as text.
Actual: as expected. `ls -A` of the root (23 entries), `MbAx` (14 with
`.DS_Store`) and `autogen` (3 folders, no file) matched the tree row for
row. The `.py` sources are not on disk in that checkout; only `.pyc` under
`__pycache__` are. `.py`, `.go`, `.rs`, `.md` and `.json` files opened as
text on the installed app.

Not a backback bug: no fix. The binary `.pyc` message is BB-BUG-030.
