# BB-GAT-005 — fix

id: BB-GAT-005
family: GAT
status: FIXED
source: `bb_crates/shell/desktop/src/bb_explorer/panels/tests_panel/actions/scan/tests.rs`

## The fix that holds

Before Enter the test reads the state once (no project open); it waits for
the search to settle only after the step that starts it.

## Proof

The filter tests pass (4) in 0.6 s; dropping the found projects from the
search's queue turns the window test red.
