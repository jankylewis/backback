# BB-LSP-008 — A killed server was reported stopped after one second under load

id: BB-LSP-008
family: LSP
status: FIXED
date: 2026-11-11
signature: a_dead_server_stops_within_one_second_and_restart_answers_again ... stopped within one second: Elapsed(())
source: `bb_crates/shell/desktop/src/intel/bb_worker/tests/bb_death.rs:25`
observed: CI run 42966229802 (PR #107, 575 tests, about 10 parallel workers): FAIL after 10.98 s; the same test alone on master passed 3 of 3 (about 9.2 s each)
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

The test kills the Roslyn child (`Request::KillForTest`) and requires the
worker's `Event::Stopped` within one second. On a CI runner busy with ten
test processes the event came later. Different from BB-LSP-001 (the
readiness wait), which this test also hit once.

## Cause (BBEOD35, measured on master 6dc536f)

The test's clock, not the product. The one second held the kill itself:
`stop_child` ended the server's group with TERM, waited the whole 200 ms
grace (Roslyn's group was not gone within it), then KILL, each step a
`/bin/kill` spawn that a loaded machine slows. Timed alone: kill to
`Stopped` 234 and 248 ms, all of it `stop_child`. The product's detection
(stdout EOF, `closed` signal, `Stopped`) took under 1 ms after the kill.

## How to confirm

Time the instrument kill apart from the detection: an `eprintln!` of the
elapsed time around `kill_child` in `bb_loop`, and kill to `Stopped` in
the test. The fix is in `failfix/APP/LSP/S02/` (same name).
