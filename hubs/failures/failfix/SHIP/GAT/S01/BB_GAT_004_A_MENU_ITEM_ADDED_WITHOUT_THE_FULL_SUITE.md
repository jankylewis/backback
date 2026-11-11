# BB-GAT-004 — fix

id: BB-GAT-004
family: GAT
status: FIXED
source: `bb_crates/shell/desktop/src/app/bb_launch/tests.rs`

## The fix that holds

The test now expects 8 View items and names Word Wrap among them, so a
lost item fails by name, not only by count.

## Proof

The menu test and the full gate green on the word wrap branch.
