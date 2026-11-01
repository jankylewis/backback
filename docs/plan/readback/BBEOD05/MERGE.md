# BBEOD05-MERGE readback

Date: 2026-11-01. Seat: `backback_2`. Branch: `bbeod-05-ship`.
Merged `origin/master` `0628b87` into debugger ship base `bb1d051`.

## Conflict resolution

| File | Kept |
|---|---|
| `bb_crates/Cargo.lock` | `bb-dap` keeps Tokio and the shared `thiserror 2.0.21` lock entry. |
| `bb_crates/shell/bb_cli/src/bb_commands/mod.rs` | Debug, scan, audit, probe, and test commands all dispatch. Roslyn commands live under `bb_intel/`. |
| `bb_crates/shell/bb_cli/src/cli/mod.rs` and `tests.rs` | Debug, probe, and test parsing and their tests all remain. |
| `docs/plan/PROGRESS.md` | Current BBEOD05 state; master history moved to `HISTORY.md` to stay within 100 lines. |
| `docs/plan/bench/BBEOD02/` | Master names `bb_BBEOD02.md` and `BBEOD02_FIX.md` win; content matches branch copies. |
| `docs/plan/readback/BBEOD02_FIX/bb_BBEOD02_FIX.md` | Link points to master bench name. |

The merged taxonomy needed three more folder fixes. Roslyn CLI commands now
share `bb_intel/`; older bench records live under `bb_GENERAL/`; the BBEOD01
readback and guide live under `bb_BBEOD01/`. The self-audit failed on three
folder counts before these moves and then passed all three tests.

## Gates

- `cargo fmt --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- Stock adapter: `cargo test --workspace --all-features -- --test-threads=2`:
  267 passed, 0 failed, 0 ignored across 41 suites.
- Patched adapter: same command: 267 passed, 0 failed, 0 ignored across 41 suites.
- `cargo deny check`: exit 0; advisories, bans, licenses, and sources okay.
  Duplicate dependency warnings remain.
- `bb audit ..`: clean. The tracked absolute-path guard passed in both gates.

## Served readbacks

- Patched adapter Set Next: line 14 stopped, `gotoTargets` chose line 12,
  `goto` stopped, two `next` steps returned through line 14, and `count`
  evaluated to `4`. The live instrument passed, 1/1, with no ignored test.
  It also rejected cross-method and exception-region targets and a reused ID.
- Stock `bb debug` on a scratch copy of `GotoFixture.csproj`: breakpoint at
  `_Program.cs:14` stopped, `count = 2` appeared, and output was `2`, `3`,
  `4`, `4`. It printed `goto: hidden`, `exit code: adapter word`, and
  `exited: code 0 (adapter-reported, unverified; ...)`.
- Test Explorer GPUI: `test_panel_lists_runs_a_class_and_opens_a_failure`
  passed in both full gates. It lists, runs a class, checks outcomes, and
  opens the failed spec at the source line.

## `runInTerminal` research

No support in the checked netcoredbg source. `rg runInTerminal` found no
reference. `src/protocols/vscodeprotocol.cpp:594-627` handles `launch` by
calling `sharedDebugger->Launch` directly. `src/debugger/manageddebugger.cpp:748-753`
creates the child through `CreateProcessForLaunch`. The source has no reverse
`runInTerminal` request path that lets backback own the child. This is a source
finding only; no protocol probe was run for this question.

## Coordination

`agentfarm claim backback_2 <absolute worktree path>` refused with exit 2:
`absolute path not allowed`. The isolated worktree was already assigned by
the brief. No claim bypass or fleet state edit was made.
