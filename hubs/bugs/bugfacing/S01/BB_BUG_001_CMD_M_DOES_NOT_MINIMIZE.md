# BB-BUG-001 — Cmd+M does not minimize the window

id: BB-BUG-001
family: UIX
status: FIXED
date: 2026-11-01
signature: Cmd+M does nothing
source: master 7b0f241: no `minimize` binding and no Window menu in `bb_crates/shell/desktop/src`
observed: headmate report: Cmd+M hides other Mac apps; in backback only the yellow button works
links: fix=yes lesson=no evidence=yes toolgap=none

## Steps

1. Open backback. 2. Press Cmd+M.

Expected: the window minimizes to the Dock, as in every Mac app.
Actual: nothing happens.

## Note

Likely missing, not broken: the app never registered the standard Window menu
(Minimize Cmd+M, Zoom) or Hide (Cmd+H). Not proven until the fix's red test.
Goal: BBEOD10.
