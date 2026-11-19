# BB-GAT-003 — fix

id: BB-GAT-003
family: GAT
status: FIXED
source: `bb_crates/shell/desktop/src/app/script/mod.rs` (`wait:tests`), `shell/desktop/tests/bb_native_discovery.rs`

## The fix that holds

A `wait:tests` script step waits (up to the script's 180 s) until every
project's discovery is done with specs listed; the test uses it in place
of `sleep:6000`. Wrong fix: a longer sleep, or a CI rerun until green.

## Proof

Green locally (6.4 s) and in the PR's CI run with parallel workers.
