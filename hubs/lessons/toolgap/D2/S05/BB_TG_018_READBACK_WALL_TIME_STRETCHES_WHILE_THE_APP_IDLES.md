# BB-TG-018 — Readback wall time stretches while the app idles

id: BB-TG-018
for: night tab loop 2026-11-04
status: OBSERVED
date: 2026-11-04
source: `bb_crates/shell/desktop/src/app/latency.rs` (`send_key`: a 30 ms timer before and after each key)

Opening 40 files and closing them with 40 Cmd+W, three rounds in one
app: 10 closes took 0.8 s in round 1 and 4.9 s in round 3 (80 -> 490 ms
a key), opening 3.7 -> 14.6 s. Per-key UI work stayed small (4.2 ms
p50); a 5 s `sample` of the main thread in a slow phase: 2825 of 2940
samples idle in the run loop. The runner's own timers fired late.

## Not proven

The cause: most likely macOS App Nap throttling the timers of a
background app nobody looks at; proving it needs the app's defaults
changed (`NSAppSleepDisabled`), which is the headmate's app.

## Workaround

Judge readbacks by `bb_keys` UI work and per-step state, not wall time
across many keys; keep long runs short or split them.

## Status

OBSERVED.
