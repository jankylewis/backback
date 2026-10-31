# BBEOD01-C docs progress

- Seat: `backback_2`; owned tree: `docs/`.
- Branch observed: `bbeod-01-engine`.
- Docs plan, conventions, and research notes are ready for handoff.
- `cargo run -q -p bb -- audit ..` returned `audit : clean (..)`.
- Source evidence: AgentFarm goals board and two BBEOD01-B spikes.

- BBEOD01-E: renamed plan marker to `bb_roadmap/`; independent frozen worktree gate pending.

## BBEOD06-A in progress (backback_2, 2026-11-01)

- Branch: `bbeod-06-mcp`, based on `978f75f`; owned worktree: `backback_wt/bbeod-06-mcp`.
- Scope: discover MCP projects, fixture-only stdio process/client, CLI, tests, readback, bench.
- Risk families: absent projects and invalid arguments (typed errors); mixed project/solution paths (distinct path types); JSON-RPC mismatched IDs and malformed frames (envelope validation tests); child exit and stuck calls (deadlines and process-tree stop); shared process state (single owner); secret-bearing stderr (do not persist live server logs).
- Initial discovery: `_Harness/McpServer` had a project and Greet tool, but no executable entry point.
- Implemented: MCP discovery, owned stdio client and process, CLI, fixture server, crash and orphan tests.
- Final gate observed: fmt, clippy, 90 tests passed, deny checks passed; `bb audit ..` clean.
- Readback: MbTx 0, NmcTx 1, SI 1 MCP projects; no real MCP launch.
- Final fixture timing: cold start to tools list 1,309 ms; warm call p50 135 µs.
- Log defect: startup stderr was lost by a late subscriber; failing test first, then fixed.
- Benchmark: final-tree fixed set ran under load1 11.28; quiet re-run remains open.
- `cargo kani` and `cargo mutants` are unavailable on this seat; the project gate does not include them.
- Handoff: implementation and gate are ready on `bbeod-06-mcp`. Quiet re-benchmark remains open because load1 stayed above 3.
