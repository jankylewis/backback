# BB-GAT-012 — fix

id: BB-GAT-012
family: GAT
status: OBSERVED
source: `messages.rs` `messages`: keeps the first of identical messages (path, line, column, level, text)

## The fix that holds

The parser lists each identical message once, whatever the job count.
`--all-targets` stays: the lib's unit-test build is the only one that
checks `#[cfg(test)]` code, so dropping it would hide those errors.

## Proof

`the_same_error_from_the_lib_and_its_test_build_lists_once`: red before
(4 messages for 2), green after; bb-cargo suites 8 + 6 + 4 passed. Not
yet proven on CI (PR #116 rerun pending), so not FIXED.

## The wrong fix

Asserting `>= 1` in the two tests, or forcing `-j1` on the Build menu:
the panel would still show the error twice on any machine with more jobs.
