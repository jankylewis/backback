# BB-BUG-004 — Test Explorer did not respond to clicks

id: BB-BUG-004
family: TST
status: OPEN
date: 2026-11-01
signature: Test Explorer click no response
observed: real-app readback by a mate on branch `bbeod-08-reap`: Run clicks in
  the Test Explorer had no visible effect; Debug was disabled
links: fix=no lesson=no evidence=no toolgap=none

## Steps

Open a solution with tests; click Run on one test in the Test Explorer.

Expected: the test runs and shows its result.
Actual: no response (one readback, driven by synthetic input).

## Not proven

Whether a person clicking by hand sees the same, and whether synthetic input
reached the window at all. Reproduce by hand first.
