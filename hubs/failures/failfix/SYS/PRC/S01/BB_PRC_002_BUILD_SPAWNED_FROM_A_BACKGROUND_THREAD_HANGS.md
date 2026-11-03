# BB-PRC-002 — fix

id: BB-PRC-002
family: PRC
status: FIXED
source: `bb_crates/shell/desktop/src/bb_explorer/panels/build/run.rs` (`start`, `wait`)

## The fix that holds

`start` spawns on the UI thread (fast); only `wait` runs in the background.
stdin is null; `-nodeReuse:false -p:UseSharedCompilation=false`; when the
build exits its process group is ended so the output pipe closes.

## The wrong fix

A longer timeout, or `nice`: the process was throttled, not slow.

## Proof

Readback: "Build failed: 1 error, 0 warnings" within seconds.
