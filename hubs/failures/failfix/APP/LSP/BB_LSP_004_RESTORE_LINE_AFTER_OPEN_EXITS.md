# BB-LSP-004 — fix

id: BB-LSP-004
family: LSP
status: FIXED
source: `bb_lsp::roslyn::wait_for_restore`; `bb open` (`finish_restore`)

## The fix that holds

With `--restore`, `bb open` keeps reading the server after the load until a
restore end line (`Restore complete`, `restore was canceled`, `restore
failed`), 120 s at most, and prints `restored  : true|false`. Order, not a
longer sleep: the restore line is in the log before shutdown on any machine.

## The wrong fix

Rerunning CI until green, or a fixed sleep after the load.

## Proof

Red first: `a_restore_wait_ends_on_the_restore_end_line_and_logs_what_came_before`
did not compile without the function. The e2e test now also asserts
`restored  : true`. CI on PR #21 failed twice on this test before the fix.
