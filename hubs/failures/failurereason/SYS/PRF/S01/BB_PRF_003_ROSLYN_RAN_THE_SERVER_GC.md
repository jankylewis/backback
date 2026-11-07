# BB-PRF-003 — Roslyn ran the server GC

id: BB-PRF-003
family: PRF
status: FIXED
date: 2026-11-04
signature: the roslyn-language-server child grows about 100 MB per editing round to about 2.4 GB
source: `bb_crates/edge/bb_lsp/src/roslyn/mod.rs` (`start`); Roslyn's own `Microsoft.CodeAnalysis.LanguageServer.runtimeconfig.json` sets `System.GC.Server: true`
observed: night soak 2026-11-04 on the installed app at `8392878`, MbTx (17 projects), `docs/plan/bench/bb_GENERAL/RERUNS/bb_NIGHT_2026-11-04.md`
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

The server GC keeps one heap per core and collects late. On a 12-core
Mac the Roslyn child grew from 0.59 to 2.1 GB in 12 editing rounds and
levelled off near 2.45 GB in a 30-round soak; 90 s idle did not give it
back. backback itself stayed flat (103 to 127 MB). Not a leak in what
backback sends: edits go as incremental `didChange`, diagnostics are
pulled only when the queue is quiet (`PULL_AFTER`).

## How to confirm it

`soak_probe.py`-style rounds with the children's RSS sampled per round;
the same run with `DOTNET_gcServer=0` levels off near 0.66 GB.
