# BB-GAT-002 — lesson

id: BB-GAT-002
family: GAT
status: FIXED
source: PR #92

## Who believed what

The tests believed "once per process" meant "once for the whole run": a
fixed folder plus a `OnceCell` was safe because one process ran them all.

## What changed the belief

Parallel CI workers (one process per test) broke it in the first run.

## Rule left behind

A test's scratch folder is owned by its process (`OwnedTempDir`, pid in
the name), never a fixed shared path; a fixture must survive any number
of test processes side by side.
