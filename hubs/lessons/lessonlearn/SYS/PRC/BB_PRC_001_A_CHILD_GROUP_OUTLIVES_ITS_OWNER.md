# BB-PRC-001 — lesson

id: BB-PRC-001
family: PRC
status: FIXED
source: `bb_crates/bb_core/process/src/bb_group.rs`

## Who believed what

Killing the process we spawned ends the work it started.

## What changed the belief

`ps` after a cancel or quit still showed the runner host and Roslyn's real
server: launchers start their own children.

## Rule left behind

Own the group, not the pid: every spawn gets its own process group in one
registry, and every exit path ends the registry.
