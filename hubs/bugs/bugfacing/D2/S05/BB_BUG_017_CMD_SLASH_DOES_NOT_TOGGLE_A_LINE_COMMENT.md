# BB-BUG-017 — Cmd+/ does not toggle a line comment

id: BB-BUG-017
family: UIX
status: FIXED
date: 2026-11-03
signature: Cmd+/ does nothing
source: sweep 2 (case G8) on the installed app at `c145669`, `docs/plan/readback/V2/SWEEP01/bb_READBACK.md`
cause: BB-UIX-008
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Put the cursor on `    int alpha = beta + gamma;`, press Cmd+/, Cmd+S.

Expected: the line reads `    // int alpha = beta + gamma;` (after the
indent, as VS Code and Rider write it); Cmd+/ again removes the comment;
with a selection every line it touches toggles.
Actual: the file is unchanged.

Goal: BBEOD02 (editor).
