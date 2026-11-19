# BB-LSP-011 — An empty rename plan opened a preview

id: BB-LSP-011
family: LSP
status: FIXED
date: 2026-11-19
signature: `Renamed: 0 edits in 0 files`
source: `bb_crates/shell/desktop/src/intel/glue/answers/mod.rs` (`Answer::Rename`); BB-BUG-079
observed: real pyright 1.1.414, F2 1.8 s after the window opened: its log stopped before it had found the source files, and the answer held no edit
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

`Answer::Rename` always made a `RenamePreview`, even with no change.
pyright answers a rename with no edit while it still indexes; Roslyn
answers one for a name it cannot rename. Enter then "applied" nothing.

## How to confirm

Answer an F2 with an empty plan (fake server) and read the status.
