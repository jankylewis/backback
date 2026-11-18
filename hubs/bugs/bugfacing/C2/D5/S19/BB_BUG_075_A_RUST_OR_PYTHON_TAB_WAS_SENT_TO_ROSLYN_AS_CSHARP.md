# BB-BUG-075 — A Rust or Python tab was sent to Roslyn as C#

id: BB-BUG-075
family: LSP
status: FIXED
date: 2026-11-19
signature: a `.rs` or `.py` tab in a window with C# active reaches Roslyn (`didOpen` languageId `csharp`)
source: found by the BBEOD27 routing test; `intel/glue/mod.rs` `send` on `feat/lang-picker` used the one Roslyn link for every tab
cause: one server link for every file; no rule chose a server by the file's language
links: fix=yes lesson=no evidence=yes toolgap=none

## Steps

Open a folder with C# and Rust ticked; open `src/lib.rs`; type; press F12.

Expected: Roslyn gets nothing about the Rust file.
Actual: Roslyn gets `didOpen`, `didChange` and the F12 question as C#
(not proven to show wrong squiggles in the app: no live readback).
