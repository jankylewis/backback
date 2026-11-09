---
name: cda-backback-perf
description: "Make backback faster with no regression allowed: fixed benchmarks, before/after medians, profile, one change at a time."
metadata:
  source: skillhub
---

# cda-backback-perf

Make backback faster with no regression allowed: fixed benchmarks, before/after medians, profile, one change at a time.

- Read [docs/plan/bench/README.md](../../../docs/plan/bench/README.md) and the latest record under `docs/plan/bench/`.
- Quiet machine (no gate, no other build), fixed case set, warm-up, five samples, median; the baseline comes from master on the same machine the same day.
- Profile before changing anything; change one thing; re-bench; keep it only when faster (>5% slower anywhere is a regression) with all tests green.
- Write a dated record: machine and OS, commit, cases, samples, medians, percent change, profile evidence, the gate result; put before/after numbers in the PR.
- Ship through cca-backback-ship; a regression found later becomes a `BB-PRF-nnn` failure in hubs.
- Load with `agentfarm skillload <seat> cda-backback-perf`.
- Codex consumes this variant: run long steps (gate, CI wait, install) as separate commands and read their exit codes; never chain a commit after a grep.

## Guardrails

- No regression allowed: a trade that slows another case is refused, not averaged away.
- Never bench on a loaded machine or against a stale baseline; say so instead of quoting the number.
- Only stop processes this session started, one pid at a time; never `pkill`, never the user's running backback.
