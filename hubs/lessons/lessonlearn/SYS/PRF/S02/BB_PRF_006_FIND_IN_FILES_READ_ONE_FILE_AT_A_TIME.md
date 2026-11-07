# BB-PRF-006 — lesson

id: BB-PRF-006
family: PRF
status: FIXED
source: perf epic, 2026-11-07

## Rule left behind

Speed up a search without changing what it finds: keep a plain
reference in the tests and compare every case, including the cap. Bench
every shape (full, no match, capped): a win on one can be a loss on
another, as the first parallel walk was for the capped search.
