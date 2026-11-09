# BB-BUG-027 — backback leaves temp folders behind

id: BB-BUG-027
family: PRC
status: FIXED
date: 2026-11-03
signature: the OS temp folder grows by a folder per app launch and per test run
source: the headmate's leak epic 2026-11-03 ("no resource leakage allowed in my machine")
cause: BB-PRC-004
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Use backback, run the gate a few times, then list `$TMPDIR`.

Expected: no folder of a process that has ended.
Actual (2026-11-03, this Mac): 3156 dead folders, 39.5 GB: `bb-e2e-test-*`
20 GB, `bb-testkit-e2e-*` 17.1 GB, `backback-testhost-*` 1.2 GB,
`bb-intel-*` / `bb-gui-*` thousands of small ones, and 544
`backback-intel-<pid>` from app launches (63 MB). Plus 74 `bb_gate.*` logs.
