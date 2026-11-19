# BB-GAT-008 — lesson

id: BB-GAT-008
family: GAT
status: FIXED
source: BBEOD25 gate log

## Who believed what

Audit clean and the targeted tests green meant the gate would pass.

## What changed the belief

Clippy, which the audit does not replace, denied two test arms.

## Rule left behind

Before the gate: fmt, clippy `-D warnings` on the touched crates, audit,
then the gate (cca-backback-develop order).
