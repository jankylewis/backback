# BB-BUG-073 — fix

id: BB-BUG-073
family: DAP
status: FIXED
source: branch `fix/five-debug-window`; `view/bb_panel/stale.rs` (`notice_row`), `view/notes.rs` (status at a stop)

## The change

A stale paused file puts `file changed since the debug build — restart to rebuild` atop VARIABLES and in the status at each stop. In a saved, built async spec a local after an await is listed (proven live).

## Lesson

Same as BB-BUG-072.

## Proof

`a_file_changed_since_the_debug_build_says_so` (the `vars-stale` row and the status); `set_next_moves_up_inside_an_async_spec_and_its_late_local_shows` lists `x` = 1 after an await.
