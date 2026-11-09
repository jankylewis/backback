# BB-BUG-016 — Cmd+A does not select all

id: BB-BUG-016
family: UIX
status: FIXED
date: 2026-11-03
signature: Cmd+A does nothing
source: sweep 2 (case G1) on the installed app at `c145669`, `docs/plan/readback/V2/SWEEP01/bb_READBACK.md`
cause: BB-UIX-008
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

In any file press Cmd+A, type `x`, Cmd+S.

Expected: the whole text is selected and the typed char replaces it, as in
every macOS text field.
Actual: nothing is selected; `x` goes in at the cursor.

Goal: BBEOD10 (standard macOS keys).
