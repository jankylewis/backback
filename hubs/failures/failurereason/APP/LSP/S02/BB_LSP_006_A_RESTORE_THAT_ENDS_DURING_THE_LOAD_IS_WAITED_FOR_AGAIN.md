# BB-LSP-006 — A restore that ends during the load is waited for again

id: BB-LSP-006
family: LSP
status: FIXED
date: 2026-11-03
signature: restored  : false after 120
source: `bb_crates/edge/bb_lsp/src/roslyn/bb_load.rs` (`wait_for_load`, `wait_for_restore`); `bb open --restore`
observed: local gate 2026-11-03 on branch `bb-dap-exception-breaks`: `open_with_restore_flag_lets_roslyn_restore` FAILED after 130 s; alone, the test passed in 5.6 s
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

`wait_for_load` reads and logs every server message until the last project
has loaded; `wait_for_restore` then reads on until a restore end line. When
Roslyn finishes the restore before the last load line (the run log shows
`Restore complete` between two batches of `Successfully completed load`),
the load wait consumes the end line, and the restore wait reads 120 s of
nothing, then says `restored  : false`. Order, not time: the restore took
48 to 161 ms per project.

## How it shows

The gate test fails now and then with `restored  : false after 120.0s`
while its own run log ends in `Restore complete` and `Completed (re)load of
3 project(s)`. Alone, the test passes.

## How to confirm it

Open the run log named in the test output: a `Restore complete` line above
the last `Successfully completed load` line is this case.
