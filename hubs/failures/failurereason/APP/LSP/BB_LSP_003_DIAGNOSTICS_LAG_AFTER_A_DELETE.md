# BB-LSP-003 — Problems still counted a deleted error 6 s after the edit

id: BB-LSP-003
family: LSP
status: OPEN
date: 2026-11-01
signature: problems=18 after backspace
source: `docs/plan/readback/FIXES/BBEOD03/STATE_LOG.txt` (run 1)
observed: 1 of 3 runs where the typing landed; load ~8-10
links: fix=no lesson=no evidence=yes toolgap=none

## What it is

After typing `zzz` (12 -> 18 problems) and deleting it, the count stayed 18
for at least 6 s in one run. In two other runs it went back to 12 within 6 s.

## Not proven

Whether the list was stale (an old version shown) or the server was slow, and
whether it would have settled: that run had no later check.

## How to confirm

Repeat the readback with `state` every second for 30 s after the deletion,
and log the document version of the shown diagnostics.
