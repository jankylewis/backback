# BB-LSP-001 — Roslyn tests wait on a fixed wall and count reloads twice

id: BB-LSP-001
family: LSP
status: OBSERVED
date: 2026-11-01
signature: before ready: 
source: `bb_crates/shell/desktop/src/intel/bb_worker/tests/mod.rs:129`
source: branch `bbeod-03-diagver` commit `28699aa` (not on `master`)
observed: 2026-11-01 local gate at load 16: `a_dead_server_stops_within_one_second_and_restart_answers_again` FAILED; rerun green
links: fix=yes lesson=yes evidence=no toolgap=none

## What it is

Two causes. A project reload sends a second batch that was counted again, so
"all projects loaded" came early. Tests also waited a fixed 30 s, but a busy
machine needs about 42 s for the probe.

## How it shows

Roslyn worker tests fail under load and pass on rerun.

## How to confirm

Run the worker tests while another `cargo test` runs; watch the project count.
