# BB-LSP-013 — A restart sent every tab to every server

id: BB-LSP-013
family: LSP
status: FIXED
date: 2026-11-20
signature: `Rust server: document is already open: …` (or `Python server: …`) after C# > Restart Server
source: `bb_crates/shell/desktop/src/intel/glue/bb_route.rs` (`restart`, `resync_open`)
observed: BBEOD39 window test on real Roslyn, rust-analyzer 1.99.1 and pyright 1.1.414: after Roslyn stopped, Cmd+Shift+R sent `src/main.rs` to rust-analyzer and `tool/main.py` to pyright a second time
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

`resync_open` sent a `didOpen` for EVERY tab, routed by file. Since
BBEOD27/28 a Rust or Python tab routes to its own server, which was still
running and already held the file open, so the second open failed
(`IntelError::AlreadyOpen`) and the status bar said so. It also took the
pending open/close events and char edits of every tab, so those of other
servers' tabs were dropped instead of sent.

## How to confirm

`a_roslyn_restart_reopens_only_the_csharp_tabs`: with all three real
servers, cut Roslyn's link, deliver `Stopped`, press Cmd+Shift+R and count
the `didOpen`s per server start on the recording seam.
