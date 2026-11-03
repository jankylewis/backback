# BB-BUG-034 — A save overwrites another tool's change

id: BB-BUG-034
family: UIX
status: FIXED
date: 2026-11-04
signature: a line another tool wrote is gone after Cmd+S in backback
source: night bug loop 2026-11-04, readback on the installed app at `8392878` (scratch file)
cause: BB-UIX-015
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Open a file; let another tool change it (git pull, a formatter, another
editor, an agent); type in backback; Cmd+S.

Expected (VS Code): the save is stopped and the user chooses: overwrite
or reload.
Actual: the save wrote backback's text over the change, without a word:
the disk read `xclass A ...`, the line `// written by another tool` lost.
