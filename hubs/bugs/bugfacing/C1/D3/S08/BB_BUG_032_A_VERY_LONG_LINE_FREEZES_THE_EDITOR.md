# BB-BUG-032 — A very long line freezes the editor

id: BB-BUG-032
family: PRF
status: FIXED
date: 2026-11-04
signature: a minified file (one 229 000-char line) takes 11 s to show and about 31 s a key
source: night robustness sweep 2026-11-04 (`robust_sweep.py`, branch build)
cause: BB-PRF-004
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Open a file with one very long line (minified JS, a generated file, a
log) and move the cursor on it.

Expected: the file shows in about a second; keys answer at once.
Actual: 11.3 s to the first state; goto plus one Right, Option+Right or
Cmd+Right took 42 s each (about 31 s for the key).
