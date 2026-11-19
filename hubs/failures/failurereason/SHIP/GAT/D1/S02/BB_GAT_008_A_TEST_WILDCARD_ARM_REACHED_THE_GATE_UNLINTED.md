# BB-GAT-008 — A test wildcard arm reached the gate unlinted

id: BB-GAT-008
family: GAT
status: FIXED
date: 2026-11-18
signature: `wildcard match will also match any future added variants` ... `clippy rc=101` with `tests: ... failed=0`
source: `session/tests/bb_async.rs` (`other => panic!`) on branch `fix/five-debug-window`
observed: self-detected: BBEOD25 gate, clippy red, 338 tests green
links: fix=yes lesson=yes evidence=no toolgap=none

## What it is

`wildcard_enum_match_arm` is denied for test code too; a bare `other =>`
on the crate's own `Note` enum compiles and passes `bb audit`, so only
clippy catches it, and clippy ran first in the gate.

## How to confirm it

`cargo clippy -p backback --all-targets -- -D warnings` before the gate.
