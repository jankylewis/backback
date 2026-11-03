# BB-LSP-006 — lesson

id: BB-LSP-006
family: LSP
status: FIXED
source: branch `bb-lsp-006-restore-race`

## Who believed what

Roslyn restores after the load, so a wait placed after the load wait would
always see the restore end line (BB-LSP-004's fix).

## What changed the belief

A run log with `Restore complete` between two load batches: Roslyn restores
as soon as the projects it has need it, and the last project can load later.

## Rule left behind

Two waits on one message stream share one reader state: whatever the first
wait consumes, the second learns from it instead of waiting for it again.
