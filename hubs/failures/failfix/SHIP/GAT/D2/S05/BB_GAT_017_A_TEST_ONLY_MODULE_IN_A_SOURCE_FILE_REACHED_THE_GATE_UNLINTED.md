# BB-GAT-017 — fix

id: BB-GAT-017
family: GAT
status: FIXED
source: commit 708d624 (PR #124), `bb_crates/shell/desktop/src/app/bb_launch/open/langs.rs:178-201`, `tests/early_walk.rs:20`

## The fix that holds

The recorder became top-level items at the end of `langs.rs`, each with
its own `#[cfg(test)]` on the line above (`WALKS`, `record_walk`,
`walks_of`): no test module sits before production items, and the map
check skips each head by the line-above rule. The helper takes the last
window with `.next_back()`.

## Proof

BBEOD36 gate run 2 (`--full`): fmt, clippy, test, deny, sweep and audit
rc=0, suites=67 passed=730 failed=0; run 3 after the rebase on c1cdf87:
suites=67 passed=733 failed=0. PR #124 CI `gate` pass; merged as 243c495.

## The wrong fix

`#[allow(clippy::items_after_test_module)]`, renaming the module to
`tests` so the map check skips the whole file (it would then skip every
real public head in it), or teaching the map check to skip any
`#[cfg(test)]` module without a test that the map lists what it skips.
