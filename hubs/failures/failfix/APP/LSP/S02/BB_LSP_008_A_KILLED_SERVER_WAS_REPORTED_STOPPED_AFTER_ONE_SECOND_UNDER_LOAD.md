# BB-LSP-008 — fix

id: BB-LSP-008
family: LSP
status: FIXED
source: branch `fix/open-hub-items` (BBEOD35)

## The fix that holds

The instrument kill ends the server as a crash would:
`LspClient::kill_child` (was `stop_child`) sends KILL to the server process
alone and reaps it; the group still ends when the client drops (`_group`),
so no descendant outlives the session (`dropping_roslyn_reaps_its_descendant`).
`Session::kill_child` and `Request::KillForTest` call it. The test's one
second now times what its name says: a dead server reported `Stopped`.

## The wrong fix

A longer wall in the test (2 s, 5 s): it hides a slow detection too. A
shorter grace in `bb_process`: every owned group (builds, test runs,
debuggers) would lose its time to exit cleanly, to fix a test.

## Proof

Red first: `the_instrument_kill_does_not_wait_out_the_group_grace`
(`bb_lsp/src/client_child/tests.rs`, a server whose group ignores TERM)
failed on master's `stop_child`: kill took 225.9 ms and 227.6 ms, over
`bb_process::GRACE` (200 ms). Green after: 3 of 3 in 0.01 s. The real
Roslyn test `a_dead_server_stops_within_one_second_and_restart_answers_again`:
kill to `Stopped` 12, 12 and 14 ms (was 234 and 248 ms), 3 of 3.
