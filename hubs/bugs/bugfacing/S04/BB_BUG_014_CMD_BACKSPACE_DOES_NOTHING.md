# BB-BUG-014 — Cmd+Backspace does nothing

id: BB-BUG-014
family: UIX
status: OPEN
date: 2026-11-03
signature: Cmd+Backspace does nothing
source: step-8 sweep (case C2) on the installed app at `0b6406d`, `docs/plan/readback/V2/SWEEP01/bb_READBACK.md`
links: fix=no lesson=no evidence=yes toolgap=none

## Steps

On `    int alpha = beta + gamma;` put the cursor after `= ` (col 17);
press Cmd+Backspace; Cmd+S.

Expected: the text from the line start to the cursor is deleted, leaving
`beta + gamma;`, as in every macOS text field; Cmd+Delete deletes to the
line end.
Actual: the file is unchanged.

Goal: BBEOD10 (standard macOS keys).
