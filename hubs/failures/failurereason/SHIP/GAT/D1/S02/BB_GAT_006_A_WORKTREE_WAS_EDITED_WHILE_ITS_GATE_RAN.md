# BB-GAT-006 — A worktree was edited while its gate ran

id: BB-GAT-006
family: GAT
status: FIXED
date: 2026-11-11
signature: gate output `fmt rc=0` then `clippy rc=101` on a branch whose tests had passed
source: the editor kit branch (self-detected)
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

The gate was started on the TESTS filter, clicks, hover and bracket work;
while it ran, the Cmd+N files were written into the same worktree. Clippy
then built a half-written module and failed. The verdict described neither
the committed tree nor the new one, so the run was stopped and redone.
