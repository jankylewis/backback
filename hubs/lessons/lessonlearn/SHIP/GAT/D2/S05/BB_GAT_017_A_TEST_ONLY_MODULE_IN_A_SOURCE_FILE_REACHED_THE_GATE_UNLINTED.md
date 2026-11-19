# BB-GAT-017 — lesson

id: BB-GAT-017
family: GAT
status: FIXED
source: BBEOD36 gate run 1 and READY evidence, 2026-11-20

## Who believed what

The new test red first, then green 3/3 with two mutations caught, meant
the change was ready for the gate.

## What changed the belief

The gate's clippy `--all-targets` linted the test code, and a repo-wide
check in another crate (`bb-kmap`) read the new source file; neither is
part of a targeted `cargo test -p backback` run.

## Rule left behind

Test-only helpers in a production file are top-level `#[cfg(test)]` items
at the end of the file (or live under `tests/`), never a named test
module mid-file. Before the gate: clippy `--all-targets -D warnings` on
the touched crate (BB-GAT-008) and `cargo test -p bb-kmap --test views`
when a source file gained a `pub` head.
