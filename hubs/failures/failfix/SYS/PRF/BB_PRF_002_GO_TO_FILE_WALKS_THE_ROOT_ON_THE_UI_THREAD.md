# BB-PRF-002 — fix

id: BB-PRF-002
family: PRF
status: FIXED
source: branch `bb-prf-002-goto`

## The fix that holds

The box opens at once as "Go to File (listing files...)"; `list_files` runs
on the background executor and `fill` adds the list when it ends, only if
the same box still waits for that root (`waits_for`).

## Proof

Red first: `go_to_file_opens_at_once_and_lists_files_off_the_ui_thread`
(failed: the list was already there on the UI thread). Same keys on the
50,000-file tree, release build: UI work max 204 -> 5.2 ms, p99 196 ->
5.2 ms. Real app: type `p421/sub/c017`, Enter opens `P421/Sub/C017.cs`.
