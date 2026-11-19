# BB-BUG-077 — fix

id: BB-BUG-077
family: LSP
status: FIXED
source: branch `fix/real-tools`; `intel/glue/answers/mod.rs` `apply` takes `(server, answer)`; `intel/glue/mod.rs` passes the server the event came from

## The change

The failed answer is reported with the name of the server that answered
(`Server::name`), as `SyncFailed` already was.

## Proof

`a_failed_answer_names_the_server_that_failed_not_roslyn` (window, keys:
F12 in a `.py` then a `.rs` tab, each server fails): red before with
`left: Some("C# server: request timed out")`, green after.
