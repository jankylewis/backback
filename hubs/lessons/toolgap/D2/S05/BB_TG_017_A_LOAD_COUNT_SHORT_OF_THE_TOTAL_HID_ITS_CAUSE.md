# BB-TG-017 — A load count short of the total hid its cause

id: BB-TG-017
for: gate run 2026-11-04 02:15 (branch `bb-bug-021-move-copy-lines`)
status: FIXED
date: 2026-11-04
source: `bb_crates/shell/desktop/src/intel/bb_worker/tests/events.rs` (`wait_ready`), `tests/mod.rs` (`loaded`)

`the_server_thread_counts_projects_then_syncs_edits_and_answers` failed
once in a full gate at load 6 to 10: progress ended at `(2, 3)` when
`Ready` came. `wait_for_load` returns only once every project is loaded
or failed, and `Ready` follows the last `Progress` on the same channel,
so a short count means a project Roslyn reported as failed. The test did
not print `Ready`'s failed list, so the cause was lost.

## Fix

`wait_ready` returns `Ready`'s failed projects too; the assertion prints
them. Run alone 8 times (4 with the workstation GC, 4 with
`DOTNET_gcServer=1`): 8 passed, so the GC change is not the cause.

## Status

FIXED (the evidence gap). If it recurs, the message names the project.

Paid off at 03:40: the message named `_XunitPlaywright.csproj`, which led
to the cause, BB-PRC-005 (two workers shared one temp folder).
