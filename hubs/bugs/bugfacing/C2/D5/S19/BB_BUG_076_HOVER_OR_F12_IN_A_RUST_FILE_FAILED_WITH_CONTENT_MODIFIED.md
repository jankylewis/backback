# BB-BUG-076 — Hover or F12 in a Rust file failed with content modified

id: BB-BUG-076
family: LSP
status: FIXED
date: 2026-11-19
signature: `server error -32801: content modified`
source: BBEOD29 live test `rust_analyzer_serves_the_fixture_or_the_start_names_the_install_hint` (1 of 6 runs red on the real rust-analyzer 1.99.1)
cause: BB-LSP-009
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Open a Cargo workspace; as soon as `Rust: Ready` shows, hover a call or
press F12 while rust-analyzer still settles its workspace.

Expected: the hover or the definition, once the server can answer.
Actual: the status says the request failed with `content modified`
(server error -32801); nothing is shown. Intermittent: it needs the request
to land during the server's workspace reload.
