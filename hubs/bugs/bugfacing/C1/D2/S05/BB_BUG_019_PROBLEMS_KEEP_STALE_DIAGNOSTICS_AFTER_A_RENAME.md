# BB-BUG-019 — Problems keep stale diagnostics after a rename

id: BB-BUG-019
family: LSP
status: FIXED
date: 2026-11-03
signature: problems=11 after a rename, no squiggle in the open file
source: sweep 3 (case H6) on the installed app at `c145669`, `docs/plan/readback/V2/SWEEP01/bb_READBACK.md`
cause: BB-LSP-007
links: fix=yes lesson=yes evidence=yes toolgap=BB-TG-015

## Steps

Rename a method used by a closed file (F2, the new name, Enter, Enter),
Cmd+S; wait ten seconds.

Expected: the Problems count follows the renamed code; no error about the
old name stays in the other file.
Actual: the Problems count stays at 11, none of them in the open file,
which shows no squiggle. They belong to the file the rename opened: its
first list was pulled before the server had the open file's edit, and only
the edited file was pulled again (BB-LSP-007).

## Correction (2026-11-03)

The first version of this page said a fresh load shows `problems=0`. That
reading proves nothing: runs that ask no question showed
`bb_diag_state version=None` (four of four, 7 to 11 s after Ready), so the
tab never reached the server and no list was pulled (BB-TG-015). The H6
copy was also not restored, so some of the 11 may be real NUnit errors. The
fix's readback uses a restored copy and asks a question first:
`docs/plan/readback/V2/SWEEP01/BUG_019_READBACK.md`.

Goal: BBEOD03 (code intelligence).
