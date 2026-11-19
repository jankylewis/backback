# BB-UIX-019 — The breakpoint store outlived its file

id: BB-UIX-019
family: UIX
status: FIXED
date: 2026-11-19
signature: a deleted file's path in the start's breakpoints
source: `bb_breakpoints/mod.rs` `all()` and `session/control.rs:115` at 6f40103
observed: BB-BUG-082
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

Breakpoints are kept by path and only a toggle removed them. A start
sent every stored path; nothing asked whether the file was still on
disk, so a deleted file's lines went to the adapter at every start.

## How to confirm it

Toggle a breakpoint in a temp file, delete it, call `for_session`:
the path is in `gone`, not in `sent`.
