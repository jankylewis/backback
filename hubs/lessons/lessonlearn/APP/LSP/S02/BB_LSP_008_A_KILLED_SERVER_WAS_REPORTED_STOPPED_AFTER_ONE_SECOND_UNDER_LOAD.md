# BB-LSP-008 — lesson

id: BB-LSP-008
family: LSP
status: FIXED
source: branch `fix/open-hub-items` (BBEOD35)

## Who believed what

The test's one second measured how fast the worker sees a dead server; a
late `Stopped` on a loaded runner meant either a slow exit watch or a tight
wall (OBSERVED, not proven, 2026-11-11).

## What changed the belief

Timing the two parts apart: the kill (TERM, 200 ms grace, KILL, each a
`/bin/kill` spawn) took 233-248 ms of the budget; detecting the dead server
took under 1 ms. A crash-like KILL brought the whole test to 12-14 ms.

## Rule left behind

A timing assert times only its subject: start the clock after the
instrument has done its work, or make the instrument as fast as the event it
stands for (a crash is a KILL, not a polite stop).
