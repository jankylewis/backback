# BB-GAT-003 — A fixed sleep was too short on a loaded runner

id: BB-GAT-003
family: GAT
status: FIXED
date: 2026-11-07
signature: native_window_discovers_tests_after_expanding_project ... bb_tests_state projects=1 discovered=0 loading=1
source: PR #97's CI run (10 nextest workers; first frame 2.9 s, normally 0.1-0.4 s)
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

The native window test expanded the test project, slept 6 s, then
expected discovery done. On a loaded runner `dotnet test` discovery took
longer; the product was right, the test guessed a time.
