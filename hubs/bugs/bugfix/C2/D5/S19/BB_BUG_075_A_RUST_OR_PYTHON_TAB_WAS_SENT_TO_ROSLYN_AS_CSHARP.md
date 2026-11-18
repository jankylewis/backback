# BB-BUG-075 — fix

id: BB-BUG-075
family: LSP
status: FIXED
source: branch `feat/rust`; `shell/desktop/src/intel/glue/route.rs` (`server_for`, `send`)

## The change

One router picks the server of every request by the file's language:
C# (and files of no language, as before) to Roslyn, Rust to
rust-analyzer, Python to none. Open, change, close and every question
go through it; readiness and the status text come from the same rule.

## Proof

`every_request_reaches_only_the_server_of_its_file_s_language` (window,
keys: open, type, F12 in a `.rs`, `.cs` and `.py` tab). Mutation: `send`
always on the Roslyn link (the old behaviour) makes it fail.
