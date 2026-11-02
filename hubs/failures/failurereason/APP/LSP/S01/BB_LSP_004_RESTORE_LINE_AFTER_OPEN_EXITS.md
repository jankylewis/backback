# BB-LSP-004 — `open --restore` can exit before Roslyn logs the restore

id: BB-LSP-004
family: LSP
status: FIXED
date: 2026-11-02
signature: open_with_restore_flag_lets_roslyn_restore
source: `bb_crates/shell/bb_cli/tests/bb_roslyn.rs` (`open_with_restore_flag_lets_roslyn_restore`)
observed: CI run 41329770963 (PR #15): FAILED at bb_roslyn.rs:137; the
  twin run 41329776988 on the same commit passed
links: fix=yes lesson=no evidence=yes toolgap=none

## What it is

`bb open --restore` returns once the projects load; Roslyn starts the
restore after that, so the "Running dotnet restore" line can come after the
command has exited. Order, not time: a fast runner loses the race.

## Fix (failfix `APP/LSP/BB_LSP_004_RESTORE_LINE_AFTER_OPEN_EXITS.md`)

With `--restore`, `bb open` waits for the restore to finish (its own log
line) before it reports ready; the test then holds on every machine.
