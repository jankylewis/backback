# BB-LSP-006 — fix

id: BB-LSP-006
family: LSP
status: FIXED
source: branch `bb-lsp-006-restore-race`

## The fix that holds

`Loaded` carries `restore_ended`: whether a restore end line came during the
load. `bb open --restore` then prints `restored  : true (ended during the
load)` and waits only when none came. One reader of the stream decides; no
line is read twice or lost.

## The wrong fix

A longer restore timeout (the line is already gone), or re-reading the run
log file (a second reader of the same stream).

## Proof

Red first: `a_restore_end_line_during_the_load_is_kept_in_loaded` did not
compile without the field. `open_with_restore_flag_lets_roslyn_restore`
still asserts `restored  : true`, and now holds in either order.
