# BB-GAT-007 — fix

id: BB-GAT-007
family: GAT
status: FIXED
source: process (the ship flow)

## The fix that holds

After audit-driven edits, build the tests (`cargo build -p <crate> --tests`)
before the gate. A menu or key change runs the launch tests before the gate
(BB-GAT-004); the menu test now also names "Split Terminal".

## Proof

The third gate green (249 passed) in PR #109.
