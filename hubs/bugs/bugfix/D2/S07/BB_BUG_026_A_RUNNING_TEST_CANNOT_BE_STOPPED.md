# BB-BUG-026 — fix

id: BB-BUG-026
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/bb_explorer/panels/tests_panel/view/rows/spec.rs` (`play_or_stop`), `bb_crates/shell/desktop/src/bb_explorer/panels/tests_panel/view/rows/bb_groups.rs` (`run_or_stop`), `bb_crates/shell/desktop/src/bb_explorer/panels/tests_panel/view/mod.rs` (`toolbar`)

## The change

While a spec runs, its ▶ is a ■, and the Run of its class and project
reads ■ Stop. The toolbar's ■ Stop shows only while a run is on. Each one
sets the run's stop flag; the testkit then kills the run's whole process
group (`OwnedGroup`, BB-PRC-001), so a hard stop, not a request.

## Proof

Window test `a_running_spec_and_the_toolbar_show_a_stop_that_stops_the_run`.
Readback (branch build, scratch copy of the hand-check solution, `bug024_readback.py`): a 60 s test running had `dotnet test` and `vstest.console` under
the app; 4 s after `teststop` none was left, the panel said "test run
stopped", and after quit no process named the scratch folder.
