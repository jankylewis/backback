# Evidence index

Paths only. Never copy logs with secrets, personal paths or account values.

| Id | Status | Artifact |
|---|---|---|
| BB-BLD-002 | FIXED | `docs/plan/readback/V1/BBEOD07/` |
| BB-CIP-001 | FIXED | Runs 41273164163 (red), 41325098190 (Roslyn green, Apex CLI red) and the BBEOD07-CI2 PR run |
| BB-CIP-002 | FIXED | Run 41323338161 (queued, cancelled) |
| BB-PRF-001 | FIXED | `docs/plan/bench/BBEOD02/` |
| BB-BUG-003 | FIXED | PR #9 description and its no-MCP screenshot |
| BB-BUG-005 | FIXED | `docs/plan/readback/FIXES/S02/BBEOD12/` (captures + pixel diff) |
| BB-BUG-007 | OBSERVED | `docs/plan/readback/FIXES/S02/BBEOD12/bb_READBACK.md` (End did not move) |

Capture next time: the build time of the binary that was read back.
| BB-LSP-003 | OPEN | `docs/plan/readback/FIXES/S01/BBEOD03/STATE_LOG.txt` |
| BB-BUG-001 | FIXED | `docs/plan/readback/FIXES/S01/BBEOD10/WINDOW_ON_SCREEN.txt` |
| BB-BUG-007 | FIXED | `docs/plan/readback/FIXES/S01/BBEOD10/STATE_LOG.txt` |
| BB-PRC-001 | FIXED | `docs/plan/readback/FIXES/S01/BBEOD08/QUIT_CHECK.txt` |
| BB-LSP-004 | FIXED | CI runs 41329770963 (red) and 41329776988 (green), same commit |
| BB-BUG-006 | FIXED | `docs/plan/readback/FIXES/S02/BBEOD13/FINDER_LAUNCH.png` |
| BB-BUG-002 | FIXED | `docs/plan/readback/FIXES/S02/BBEOD14/bb_READBACK.md` |
| BB-PRC-002 | FIXED | `docs/plan/readback/FIXES/S02/BBEOD13/PART2_STATE_LOG.txt` |
| BB-PRF-002 | FIXED | `docs/plan/bench/BBEOD06/GOTO.md` |
| BB-LSP-005 | FIXED | CI run 41501823925 (red, PR #30) |
| BB-CIP-003 | FIXED | CI run 41510600740 (red, PR #33) |
| BB-CIP-004 | FIXED | Runs 42079599220 (no runner, then rerun green) and 42079606160 (green), PR #88 |
| BB-GAT-001 | FIXED | CI run 41532817663 (red, PR #39) |
| BB-GAT-002 | FIXED | CI run 42191585967 (7 bb-dap red, 4 workers); local repro 5/11 red, after fix 40/40 |
| BB-GAT-003 | FIXED | PR #97 CI run: `discovered=0 loading=1` after a 6 s sleep, first frame 2.9 s |
| BB-GAT-004 | FIXED | Local gate: `menus_go_app_then_file_then_view_as_on_every_mac` 8 vs 7, suites=53 passed=558 failed=1 |
| BB-CIP-005 | FIXED | PR #97: `OPEN` printed, worktree removed; restored from `daf397d` on origin |
| BB-PRC-007 | OBSERVED | PR #93 CI run: nextest `LEAK` on `the_server_thread_counts_projects_then_syncs_edits_and_answers` |
| BB-PRF-005 | FIXED | MbTx process tree at idle (launcher 49 MB); 3+3 runs; `bb probe` hover x10 A/B |
| BB-PRF-006 | FIXED | MbTx `find_bench` A/B, 3 rounds x 7 runs, 4 queries; `tests/parallel.rs` |
| BB-BUG-010 | FIXED | Byte dumps before/after in `hubs/bugs/bugfix/C1/D1/S03/BB_BUG_010_*.md` |
| BB-BUG-011 | FIXED | Sweep bytes before the fix in `hubs/bugs/bugfacing/C1/D1/S03/BB_BUG_011_*.md` |
| BB-BUG-012 | FIXED | Sweep states before the fix in `hubs/bugs/bugfacing/C1/D1/S03/BB_BUG_012_*.md` |
| BB-DAP-001 | FIXED | The raw `exceptionInfo` reply in `docs/plan/readback/V2/BBEOD19/bb_READBACK.md` |
| BB-LSP-006 | FIXED | Gate log 2026-11-03 (`bb_gate.cHCf6V358z`, local temp): `restored : false after 120.0s` with `Restore complete` in the run log; mechanism quoted in the reason page |
| BB-BUG-008 | FIXED | `docs/plan/readback/FIXES/S04/BB_BUG_008/bb_READBACK.md` |
| BB-BUG-009 | FIXED | `docs/plan/readback/FIXES/S04/BB_BUG_009/bb_READBACK.md` |
| BB-ENV-001 | FIXED | The BB-BUG-008 and BB-BUG-009 readbacks above |
| BB-UIX-004 | FIXED | The BB-BUG-010 byte dumps above |
| BB-UIX-005 | FIXED | The BB-BUG-011 and BB-BUG-012 sweeps above |
| BB-BUG-013 | FIXED | `docs/plan/readback/V2/SWEEP01/bb_READBACK.md` (the headmate's screenshot described; `state` with `scroll_x` on the real file) |
| BB-UIX-006 | FIXED | The BB-BUG-013 readback above |
| BB-BUG-014 | FIXED | Sweep C2 bytes before, the rerun after, in `docs/plan/readback/V2/SWEEP01/bb_READBACK.md` |
| BB-BUG-015 | FIXED | Sweep E1 and E2 bytes before, the rerun after, in `docs/plan/readback/V2/SWEEP01/bb_READBACK.md` |
| BB-UIX-007 | FIXED | The BB-BUG-014 and BB-BUG-015 sweeps above |
| BB-BUG-016 | FIXED | Sweep 2 G1 bytes before, the rerun after, in `docs/plan/readback/V2/SWEEP01/bb_READBACK.md` |
| BB-BUG-017 | FIXED | Sweep 2 G8 bytes before, the rerun after, in `docs/plan/readback/V2/SWEEP01/bb_READBACK.md` |
| BB-UIX-008 | FIXED | The BB-BUG-016 and BB-BUG-017 sweeps above |
| BB-BUG-018 | FIXED | Sweep 3 H6 bytes before (`HalfHalve`) and after, in `docs/plan/readback/V2/SWEEP01/bb_READBACK.md` |
| BB-BUG-019 | OPEN | Sweep 3 H6 states in `docs/plan/readback/V2/SWEEP01/bb_READBACK.md`: `problems=11` three times after the rename, `0` on a fresh load; `dotnet build` clean |
| BB-UIX-009 | FIXED | The BB-BUG-018 sweep above |
| BB-BUG-061 | FIXED | Reproduced outside the app: the pinned `apex` tool on a listing stub printed `dotnet test <stub> ... --no-build` then MSB4057; `apex --help` lists no `--list`, `--json` or `--specs` (bugfacing entry) |
| BB-BUG-062 | FIXED | Window test `the_active_tab_scrolls_into_sight_and_the_wheel_brings_the_first_back`: red on master (active last tab out of sight), green after; headmate screenshot 2026-11-10 |
| BB-GAT-005 | FIXED | Local run: `the search did not end` after 60 s, then 4 passed in 0.55 s with the read moved before Enter |
| BB-GAT-006 | FIXED | Stale gate: `fmt rc=0`, `clippy rc=101` after mid-run edits; the rerun on the finished tree is in PR #103 |
| BB-ENV-002 | FIXED | Headmate report 2026-11-11 mid-gate; 1-worker gates green at 578 and 581 passed |
