# BB-PRC-002 — lesson

id: BB-PRC-002
family: PRC
status: FIXED
source: `bb_crates/shell/desktop/src/bb_explorer/panels/build/run.rs`

## Who believed what

Spawning a child from a background executor is free: only the wait is long.

## What changed the belief

Five controlled runs outside the app all built in 2 s; the only difference
left was the spawning thread's QoS (`SN` in `ps`).

## Rule left behind

Spawn tools on the UI (or a normal-QoS) thread; wait in the background.
