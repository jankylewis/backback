# BBEOD01-C docs progress

## BBEOD05-A debugger engine in progress

- Seat: `backback_2`; branch: `bbeod-05-debug`; base: `978f75f`.
- Official netcoredbg `3.2.0-1092` macOS arm64 archive passed its published SHA-256 check.
- Next: typed DAP session, real debugger probe, gate, readback, and benchmark evidence.
- DAP attach readback: GreeterSpec stopped, locals were visible, step resumed, and both tests finished.
- Probe CLI stopped at a temporary console fixture and printed stack and locals.
- Measured attach-to-first-stop 594 ms and step-to-stop 19 ms in one real run.
- Final gate: format, Clippy, 87 tests, and cargo-deny all passed.
- Performance: five-run release benchmark completed at load1 5.02; quiet comparison remains open.
- Remaining: handoff summary, commit, and READY to `backback_mm`.

## BBEOD05-A acceptance benchmark

- Functionally accepted; performance comparison is in progress.
- A: master `978f75f`; B: `bbeod-05-debug` `e01277d`.
- Plan: release builds, five interleaved A/B pairs, three measured runs per item.
- Target items: scan-harness, scan-solution, and audit-repo.
- The redundant test-only Clippy allow was removed; `#![cfg(test)]` kept the integration helpers in test scope.
- A/B second window: median paired B deltas +2.7% harness, −1.5% solution, +3.8% audit.
- A/A control and sign-changing pairs support `noise, no regression`.

- Seat: `backback_2`; owned tree: `docs/`.
- Branch observed: `bbeod-01-engine`.
- Docs plan, conventions, and research notes are ready for handoff.
- `cargo run -q -p bb -- audit ..` returned `audit : clean (..)`.
- Source evidence: AgentFarm goals board and two BBEOD01-B spikes.

- BBEOD01-E: renamed plan marker to `bb_roadmap/`; independent frozen worktree gate pending.
