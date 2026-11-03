# BB-LSP-007 — fix

id: BB-LSP-007
family: LSP
status: FIXED
source: branch `bb-bug-019-diag-refresh`

## The fix that holds

Every change marks every open document (`mark_open`). A refresh request
(`workspace/diagnostic/refresh`) or a project (re)load line in the server
log does the same, through `Session::refresh_signal`. The capability goes
out as JSON under the protocol's name.

## The wrong fix

Pulling only the other files of the rename plan: the next cross-file edit,
a restore or a generated file goes stale the same way. A longer wait before
the pull: the list is not late, it is never asked for.

## Proof

Red first: `an_edit_in_one_file_refreshes_the_diagnostics_of_the_other_open_files` on master's server loop: fails after 94.6 s (`the spec never got a list where the call to the renamed Hello fails within 90 s`). Unit: the capability
JSON (`initialize_declares_diagnostic_refresh_support_under_the_protocol_name`),
the reload line (`a_reload_batch_line_is_a_project_reload_and_a_load_line_is_not`)
and the refresh decision
(`the_refresh_request_and_a_reload_line_ask_for_a_refresh_and_the_rest_do_not`).
