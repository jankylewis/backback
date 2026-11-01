# BB-LSP-002 — A test takes the first event after Open as the diagnostics

id: BB-LSP-002
family: LSP
status: FIXED
date: 2026-11-01
signature: trace.rs:38:5
source: `bb_crates/shell/desktop/src/intel/bb_worker/tests/trace.rs:38` (before the fix)
source: same assert at `tests/mod.rs:129` and `tests/bb_death.rs:36`
observed: BBEOD12 gate, 2026-11-01 23:05, load ~10:
  `gui_solution_open_and_document_sync_leave_no_trace` FAILED at trace.rs:38;
  273 other tests passed
related: BB-LSP-001 (late project-load batches after Ready)
links: fix=yes lesson=no evidence=no toolgap=none

## What it is

Three worker tests asserted that the very next event after `Open` is
`Diagnostics`. On a slow load the server can still send a `Progress` batch
first, so the assert fails though the server is right. The assert did not
print which event came, so the exact event of this run is not proven.

## How to confirm

Run the worker tests while another `cargo test` runs; on failure the new
helper prints the event it got.
