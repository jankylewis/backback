# BB-TG-015 — A readback that asks no question reads state the server never got

id: BB-TG-015
for: BB-BUG-019
status: FIXED
date: 2026-11-03
source: four probe runs on the installed app at `c145669`, `docs/plan/readback/V2/SWEEP01/BUG_019_READBACK.md`; `bb_crates/shell/desktop/src/intel/glue/mod.rs` (`flush`)

A tab's `didOpen`, and every edit, reach the server only through `flush`,
which runs on a paint (`render`) and before a question (`ask`). A BB_KEYS
readback while the display sleeps gets no paint, so a run that asks no
question reads `problems` and `bb_diag_state` for a tab the server never
opened: `version=None` and `problems=0`, which look like a clean file.

## Fix

The runner flushes after every step, as the paint after a key would
(`flush_as_a_paint`, `bb_crates/shell/desktop/src/app/latency.rs`). It runs
outside the measured key work and sends nothing when a paint or a question
already did.

## Proof

The four runs that ask no question read `version=None` on the installed
app at `e87dac2` and `version=Some(1)` on the branch build:
`docs/plan/readback/V3/TG015/bb_READBACK.md`.

## Status

FIXED. On a build older than the fix, ask a question before reading intel
state.
