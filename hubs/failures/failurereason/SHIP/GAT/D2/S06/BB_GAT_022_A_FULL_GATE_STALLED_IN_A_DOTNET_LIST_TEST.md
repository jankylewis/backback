# BB-GAT-022 — A full gate stalled in a .NET list test

id: BB-GAT-022
family: GAT
status: OPEN
date: 2026-11-28
signature: test bb_testing::list_leaves_the_built_solution_tree_unchanged ...
source: `docs/release/scripts/gate.sh --full` on PR #133 branch, 2026-11-28
source: `bb_crates/shell/bb_cli/tests/dotnet/bb_testing.rs:68-101`
observed: the test remained idle for over 7 minutes, its direct child was defunct, and the gate was stopped with exit 130
links: fix=no lesson=no evidence=yes toolgap=none

## What is proven

The full local gate passed format and Clippy, then reached the workspace
tests. The log stopped at this test. Its process used 0% CPU after more than
7 minutes, with a defunct direct child. The test has no own timeout. The gate
was stopped; no full gate verdict or test count was produced. The cause of
the wait, and whether another concurrent job affected it, are not proven.

## Next proof

Reproduce this test alone with a bounded watchdog. Capture the `dotnet`
command stage and process tree before stopping the test, then narrow the
blocking wait. Do not count the interrupted full gate as green or red.
