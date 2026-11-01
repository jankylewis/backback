# BB-BUG-007 — Home and End (and Cmd+Left/Right) do nothing in the editor

id: BB-BUG-007
family: UIX
status: OBSERVED
date: 2026-11-01
signature: end key does not move the cursor
source: `bb_crates/shell/desktop/src/editor/input.rs:185` (only `left`/`right`)
source: `docs/plan/readback/FIXES/BBEOD12/bb_READBACK.md` (Not covered)
links: fix=no lesson=no evidence=yes toolgap=none

## Steps

Put the cursor mid-line; press End (or Cmd+Right).

Expected: the cursor goes to the line end (Home / Cmd+Left: line start).
Actual: nothing; the BBEOD12 readback captures before and after are identical.

Goal: BBEOD10 (standard macOS keys).
