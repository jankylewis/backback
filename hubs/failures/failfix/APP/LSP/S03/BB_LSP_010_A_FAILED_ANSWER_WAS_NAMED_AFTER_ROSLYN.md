# BB-LSP-010 — fix

id: BB-LSP-010
family: LSP
status: FIXED
source: `answers::apply(view, pending, (server, answer), ...)`

## The fix that holds

The answer travels with the server it came from; the text is
`Server::name()`, the same rule as `SyncFailed` and the side-server
status. Sweep: the other `C# server` texts left are Roslyn-only paths
(`model/shown/status.rs`, `glue/bb_route.rs:73`).

## Proof

`a_failed_answer_names_the_server_that_failed_not_roslyn` red before
(`C# server: request timed out`), green after.

## The wrong fix

A second literal per language in `apply`.
