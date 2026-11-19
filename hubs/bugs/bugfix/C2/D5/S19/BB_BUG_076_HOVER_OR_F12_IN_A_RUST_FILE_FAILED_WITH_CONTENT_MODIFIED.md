# BB-BUG-076 — fix

id: BB-BUG-076
family: LSP
status: FIXED
source: branch `fix/real-tools`; `bb_crates/edge/intel/src/bb_sessions/bb_session.rs` (`Session::raw`)

## The change

`Session::raw`, the one funnel of every editor request for all three
servers, asks again after a -32801 `content modified` reply (200 ms pause)
while the request's deadline lasts; past it, the server's own error.

## Proof

`a_content_modified_reply_is_asked_again_and_the_hover_arrives` (scripted
server: two -32801 replies, then the hover): red before with
`Lsp(Server { code: -32801, message: "content modified" })`, green after.
`a_server_that_never_settles_ends_at_the_deadline_with_its_own_error`.
Real rust-analyzer live test 8 of 8 green after (1 of 6 red before).
