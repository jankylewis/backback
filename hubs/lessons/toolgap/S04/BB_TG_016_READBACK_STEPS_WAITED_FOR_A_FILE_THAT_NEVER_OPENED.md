# BB-TG-016 — Readback steps waited for a file that never opened

id: BB-TG-016
for: BB-BUG-029, BB-BUG-030
status: FIXED
date: 2026-11-04
source: `bb_crates/shell/desktop/src/main.rs` (`finish_open`), `bb_crates/shell/desktop/src/app/bb_launch/mod.rs` (`build_window`)

`BB_KEYS` steps started only once a file opened. A launch on a binary
file (the open fails) or on a folder (no file) never ran them: the
readback looked like a hang at 0 % CPU.

## Fix

A failed open and a folder launch start the steps too. New step
`open:<path>` opens a file under the root as a click does; `state` prints
`bb_tree_state` (selected row, row count).

## Proof

The `.pyc` launch ran its `state` step (`tab=None`, the refusal); the
folder launch ran `open:` and `state` (BB-BUG-029's readback).

## Status

FIXED.
