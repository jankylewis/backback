# BB-BUG-019 — Problems keep stale diagnostics after a rename

id: BB-BUG-019
family: LSP
status: OPEN
date: 2026-11-03
signature: problems=11 after a rename, 0 on a fresh load
source: sweep 3 (case H6) on the installed app at `c145669`, `docs/plan/readback/V2/SWEEP01/bb_READBACK.md`
links: fix=no lesson=no evidence=yes toolgap=none

## Steps

Rename a method used by a closed file (F2, the new name, Enter, Enter),
Cmd+S; wait ten seconds.

Expected: no problems: the renamed solution builds clean and a fresh load
shows `problems=0`.
Actual: the Problems count stays at 11 (none of them in the open file,
which shows no squiggle). The diagnostics are those of the closed file's
old text, pulled between the open buffer's edit and the closed file's
save, and never replaced.

## Not proven

Which request last delivered the 11, and whether a later edit in the open
file clears them.
