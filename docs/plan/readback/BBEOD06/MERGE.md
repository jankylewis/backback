# BBEOD06-MERGE readback

Date: 2026-11-01. Seat: `backback_2`. Branch: `bbeod-06-gui`.
Merged `origin/master` `3d83ae0` into MCP panel base `616fc14`. No push.

## Conflict resolution

| Conflict | Resolution |
|---|---|
| `bb_commands/mod.rs` | Kept MCP and debugger error and dispatch arms. Used master's `bb_intel::probe` path. |
| `cli/mod.rs` | Kept `mcp` and `debug` parsing, variants, errors, and help text. A regression test parses all four MCP subcommands and debug. |
| `docs/plan/PROGRESS.md` | Kept current BBEOD06 and BBEOD05 states; older BBEOD02-B detail stays in `HISTORY.md`. Both files meet the 100-line cap. |
| `docs/plan/bench/EDITOR.md` rename | Used master name `bench/bb_GENERAL/EDITOR.md`; both conflict copies had identical content. |

The first full test run failed its self-audit: `docs/plan/bench` had six
entries. This merge widened the folder. I moved `TEST_DISCOVERY` under
`bb_GENERAL` and updated its two links in `HISTORY.md`. The same self-audit
then passed all three tests. This is the folder-fanout merge family; the
repository self-audit is its independent guard. I grouped the harness
screenshots and record under `readback/BBEOD06/HARNESS/` so this new record
also stays within the five-entry limit.

## Gates

- `cargo fmt --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- Stock `BACKBACK_NETCOREDBG`, `CARGO_BUILD_JOBS=2`:
  `cargo test --workspace --all-features -- --test-threads=2` passed
  301 tests, 0 failed, 0 ignored across 46 suites.
- `cargo deny check`: exit 0; advisories, bans, licenses, sources okay.
- `bb audit ..`, tracked absolute-path guard, and `git diff --check`:
  passed after the document move.

## Served readbacks

- GPUI MCP panel test `window_mouse_start_and_keystroke_edit_reach_harness`
  passed. It started and stopped the fixture from the window view.
- GPUI Test Explorer test `test_panel_lists_runs_a_class_and_opens_a_failure`
  passed. It ran fixture specs and opened a failed spec at its source line.
- `bb mcp list` found `_McpServer.csproj`. In a scratch copy, `bb mcp tools`
  listed `Greet` and `Hold`; `bb mcp call ... Greet` returned `hi Ada`.
  The first scratch copy omitted the referenced `Core` project and failed
  its build. A complete scratch copy passed; no product source changed.
- Stock netcoredbg `3.2.0-1`: `bb debug` on a scratch GotoFixture stopped at
  `_Program.cs:14`, showed `count = 2`, printed `2, 3, 4, 4`, and exited 0.
  It showed `goto: hidden` and labeled the adapter exit code unverified.
- The Roslyn rename probe passed in the full gate; its 30-second timeout
  did not recur.

## Benchmark state

No new quiet-machine benchmark ran in this merge unit. The existing BBEOD06
fixed-set comparison is noisy; `bench/MCP/bb_BBEOD06_MCP.md` records its
medians and the open quiet re-run. No new performance delta is verified.
