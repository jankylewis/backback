# BB-CIP-003 — fix

id: BB-CIP-003
family: CIP
status: FIXED
source: branch `bb-cip-003-yanked`

## The fix that holds

`cargo update -p async-executor`: 1.14.0 -> 1.14.1 in `Cargo.lock`, one
package, nothing else moved.

## Proof

crates.io API: 1.14.0 `yanked: true`, 1.14.1 published 2026-11-02. After the
update `cargo deny check` passes locally; CI runs it with a fresh index.
