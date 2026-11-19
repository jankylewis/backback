# BB-LSP-010 — A failed answer was named after Roslyn

id: BB-LSP-010
family: LSP
status: FIXED
date: 2026-11-19
signature: `C# server: ` after a request in a `.rs` or `.py` tab
source: `bb_crates/shell/desktop/src/intel/glue/answers/mod.rs` (`Answer::Failed` arm, a `"C# server"` literal); BB-BUG-077
observed: window test with fake Python and Rust servers failing F12
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

BBEOD27/28 routed requests to three servers, but the answer handler kept
the one-server text: every failed answer said `C# server`. `on_event`
knew the server (its `SyncFailed` arm used `server.name()`), `apply` did
not receive it.

## How to confirm

F12 in a `.py` tab with a failing pyright; read the status.
