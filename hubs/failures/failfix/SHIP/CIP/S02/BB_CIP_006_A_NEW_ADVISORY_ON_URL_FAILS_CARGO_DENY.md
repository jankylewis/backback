# BB-CIP-006 — fix

id: BB-CIP-006
family: CIP
status: OBSERVED
source: branch `bbeod-45-find-panel` (PR #133)

## The fix that holds

`cargo update -p url --precise 2.5.9` in `bb_crates/`. Cargo also moved
`form_urlencoded` from 1.2.2 to 1.2.3, which `url` 2.5.9 requires.
Only `Cargo.lock` changes.

## Proof

Local `cargo deny check advisories` returns `advisories ok`, exit 0.
`cargo build -j1 -p bb` finished, exit 0 (2026-11-27). CI on the
pushed head must show the dependency audit green before this is FIXED.

## Wrong fix

Ignoring the advisory in `deny.toml`: the fixed version exists.
