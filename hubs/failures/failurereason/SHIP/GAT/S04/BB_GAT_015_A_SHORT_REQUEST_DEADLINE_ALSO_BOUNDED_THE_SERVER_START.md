# BB-GAT-015 — A short request deadline also bounded the server start

id: BB-GAT-015
family: GAT
status: FIXED
date: 2026-11-19
signature: `Lsp(Timeout { method: "initialize", after: 2s })` in `settle_tests`
source: `bb_crates/edge/intel/src/bb_sessions/servers/server_tests/settle.rs` (`hover_on`: `request_timeout: deadline`)
observed: PR #118 CI run 44444581357 (macOS, 4 workers): `a_server_that_never_settles_ends_at_the_deadline_with_its_own_error` panicked at `settle.rs:93`; green locally with 1 worker
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

The test gave its short 2 s deadline (meant for the hover under test) to
`PythonConfig::request_timeout`, which is also the client's deadline for
`initialize` (`bb_start.rs`). On a loaded runner the scripted server's
start took longer than 2 s, so the test failed before the request it was
about. The live session tests shared one 20 s constant the same way.

## How to confirm

Make the scripted server answer `initialize` after 2.5 s with the start
on the request deadline: red with the CI message, every run.
