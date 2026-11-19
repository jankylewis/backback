# BB-PRF-009 — lesson

id: BB-PRF-009
family: PRF
status: OBSERVED
source: `docs/plan/bench/bb_GENERAL/RERUNS/BBEOD37_BINARY_SIZE_AND_DNAAX.md`

## Who believed what

Starting the walk off the UI thread (BB-PRF-007) brought first frame back
to base, because MbTx and 50,000 files did.

## What changed the belief

A tree of 4,873 folders walks in 229 ms: longer than the app takes to
start, so the window still waits. Folder count, not file count, sets the
walk's cost.

## Rule left behind

A walk that overlaps the start is free only when it ends first; bench the
first open on a tree of many folders as well as one of many files.
