# BB-BLD-001 — lesson

id: BB-BLD-001
family: BLD
status: OBSERVED
source: `hubs/failures/failurereason/SHIP/BLD/BB_BLD_001_STALE_APP_RUN_FROM_A_WORKTREE.md`

## Who believed what

A screenshot of the MCP panel read as "the removal failed".

## What changed the belief

The running process started before the fix merged, from a worktree bundle.

## Rule left behind

Before judging a UI report, name the build: process start time, binary mtime,
commit. Users run one installed app built from `master`.
