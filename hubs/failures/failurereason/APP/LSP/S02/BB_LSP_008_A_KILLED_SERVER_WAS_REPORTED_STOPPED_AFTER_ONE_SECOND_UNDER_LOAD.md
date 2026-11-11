# BB-LSP-008 — A killed server was reported stopped after one second under load

id: BB-LSP-008
family: LSP
status: OBSERVED
date: 2026-11-11
signature: a_dead_server_stops_within_one_second_and_restart_answers_again ... stopped within one second: Elapsed(())
source: `bb_crates/shell/desktop/src/intel/bb_worker/tests/bb_death.rs:25`
observed: CI run 42966229802 (PR #107, 575 tests, about 10 parallel workers): FAIL after 10.98 s; the same test alone on master passed 3 of 3 (about 9.2 s each)
links: fix=no lesson=no evidence=yes toolgap=none

## What it is

The test kills the Roslyn child (`Request::KillForTest`) and requires the
worker's `Event::Stopped` within one second. On a CI runner busy with ten
test processes the event came later. Not proven: whether the worker's
exit watch is slow under load (a product delay a user could see) or only
the test's one-second wall is too tight for a loaded runner. Different from
BB-LSP-001 (the readiness wait), which this test also hit once.

## How to confirm

Run the test alone (passes), then beside a full `cargo test` or under
`nextest` with many workers; time from the kill to `Stopped`.
