# BB-BUG-082 — fix

id: BB-BUG-082
family: UIX
status: FIXED
source: branch `fix/away-run`; `Breakpoints::for_session` (`bb_breakpoints/mod.rs`), `for_start` (`bb_breakpoints/toggle.rs`), used by `begin` in `session/control.rs`

## The change

`all()` is gone. A start calls `for_start`: it drops the files no longer
on disk from the store, sends the rest, and the status names the dropped
files.

## Proof

`a_deleted_files_breakpoints_are_not_sent_and_are_dropped_and_named` red
with the old send-all rule (B.cs sent), green after;
`a_debug_start_drops_a_deleted_files_breakpoints_and_says_so` (view)
kills the no-status mutation.
