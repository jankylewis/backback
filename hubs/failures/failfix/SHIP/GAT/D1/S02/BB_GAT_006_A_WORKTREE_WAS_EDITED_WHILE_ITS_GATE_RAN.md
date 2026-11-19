# BB-GAT-006 — fix

id: BB-GAT-006
family: GAT
status: FIXED
source: process (the ship flow)

## The fix that holds

The stale run was stopped (its own pids, one at a time) and one gate ran
on the finished tree. Work that goes on during a gate goes into another
worktree.

## Proof

The rerun gate's counts in PR #103.
