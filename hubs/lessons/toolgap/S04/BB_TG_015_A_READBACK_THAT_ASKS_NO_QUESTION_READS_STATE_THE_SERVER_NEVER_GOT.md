# BB-TG-015 — A readback that asks no question reads state the server never got

id: BB-TG-015
for: BB-BUG-019
status: OBSERVED
date: 2026-11-03
source: four probe runs on the installed app at `c145669`, `docs/plan/readback/V2/SWEEP01/BUG_019_READBACK.md`; `bb_crates/shell/desktop/src/intel/glue/mod.rs` (`flush`)

A tab's `didOpen`, and every edit, reach the server only through `flush`,
which runs on a paint (`render`) and before a question (`ask`). A BB_KEYS
readback while the display sleeps gets no paint, so a run that asks no
question reads `problems` and `bb_diag_state` for a tab the server never
opened: `version=None` and `problems=0`, which look like a clean file.

## Workaround

Ask a question (F12, Shift+F12, a hover) before reading intel state, and
trust `problems` only with `bb_diag_state version=Some(..)`.

## Status

OBSERVED. The `state` step could refuse to print `problems` while the tab
has no shown version, or the runner could flush as a paint would;
proposed, not built.
