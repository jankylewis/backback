# BB-LSP-009 — A content modified reply was shown as a failure

id: BB-LSP-009
family: LSP
status: FIXED
date: 2026-11-19
signature: `Lsp(Server { code: -32801, message: "content modified" })`
source: `bb_crates/edge/intel/src/bb_sessions/bb_session.rs` (`Session::raw`); BB-BUG-076
observed: real rust-analyzer 1.99.1, live session test, hover right after definition: 1 of 6 runs red
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

LSP error -32801 (`ContentModified`) means the server dropped a request
because its state changed meanwhile; the client is expected to ask again.
rust-analyzer sends it while it reloads its workspace after the first
open. `Session::raw` passed every JSON-RPC error up as a failure, so the
window showed it. Roslyn and pyright can send it too (same funnel).

## How to confirm

A scripted server that answers -32801 first (the
`settle.rs` session tests), or the real rust-analyzer live test repeated.
