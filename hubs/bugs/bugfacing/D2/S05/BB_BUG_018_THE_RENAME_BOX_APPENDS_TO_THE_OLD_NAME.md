# BB-BUG-018 — The rename box appends to the old name

id: BB-BUG-018
family: UIX
status: FIXED
date: 2026-11-03
signature: HalfHalve
source: sweep 3 (case H6) on the installed app at `c145669`, `docs/plan/readback/V2/SWEEP01/bb_READBACK.md`
cause: BB-UIX-009
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Put the cursor on `Half`, press F2, type `Halve`, Enter, Enter.

Expected: the box opens with `Half` selected, so typing replaces it and the
symbol becomes `Halve` everywhere, as in VS Code and Rider.
Actual: the typed chars are added to the old name: the symbol becomes
`HalfHalve` in both files.

Goal: BBEOD03 (code intelligence).
