# BB-GAT-021 — A quit test waited for a missing Roslyn process

id: BB-GAT-021
family: GAT
status: OPEN
date: 2026-11-27
signature: servers missing ["Microsoft.CodeAnalysis.LanguageServer"], mark bb_mark up printed true
source: PR #133 CI run 45985075288, `no_orphan_servers` quit test
source: `bb_crates/shell/desktop/tests/orphans/run.rs:12,46-65`
observed: CI reported 775/776; this test failed after 240.440 s during setup, before sending quit
links: fix=no lesson=no evidence=yes toolgap=none

## What is proven

`up` waits up to 240 s for every wanted process. In this run it saw Rust and
Python, but never a Roslyn process. The app's `wait:ready` timed out at 180 s
and then printed `bb_mark up`; the test kept waiting for Roslyn. The quit
test's `go()` and quit assertion follow `up`, so neither ran. This is a setup
failure, not observed evidence that quitting left an orphan.

The prior PR #133 CI run 45983707640 passed this test in 15.469 s, and the
later run passed other Roslyn tests. On the same branch, the test passed alone
locally in 5.96 s and all 9 orphan scenarios passed in 46.06 s with one
test thread. Why Roslyn was absent in that one CI process remains unproven:
the app's temporary server log and process state were not retained.

## Next proof

Rerun CI at the same commit. On any repeat, retain the failing app's C#
status, server log, and process tree before the temporary fixture drops.
Then distinguish a server launch failure, a server exit, and a process-tree
observation miss. Do not call a green rerun a cause.
