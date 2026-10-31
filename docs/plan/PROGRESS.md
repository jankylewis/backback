# BBEOD01-C docs progress

- Seat: `backback_2`; owned tree: `docs/`.
- Branch observed: `bbeod-01-engine`.
- Docs plan, conventions, and research notes are ready for handoff.
- `cargo run -q -p bb -- audit ..` returned `audit : clean (..)`.
- Source evidence: AgentFarm goals board and two BBEOD01-B spikes.

- BBEOD01-E: renamed plan marker to `bb_roadmap/`; independent frozen worktree gate pending.

## BBEOD03-A (backback_1, branch `bbeod-03-intel`, worktree `backback_wt/bbeod-03-intel`)

- New crate `edge/intel` (`bb-intel`): `Session` (sync + 7 features), `bb probe`.
- Issues fixed, failing first: load signal before the load (wait for every
  project); completion `null` as the first request; references repeated the
  declaration; an edit inside a CRLF pair drifted the server text; a
  timed-out request left its waiter behind.
- Readback: 16 probes on MbTx and Apex, record `readback/2026-11-01_BBEOD03_A.md`.
- Open: references ~500 ms server floor; session drops log lines after load.
