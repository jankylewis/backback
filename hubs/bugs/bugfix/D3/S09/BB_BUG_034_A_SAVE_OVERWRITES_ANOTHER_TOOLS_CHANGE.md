# BB-BUG-034 — fix

id: BB-BUG-034
family: UIX
status: FIXED
source: `bb_crates/bb_core/bb_text/src/document/io.rs` (`DiskGuard`, `open`, `save`), `bb_crates/bb_core/bb_text/src/document/mod.rs` (`DocumentError::ChangedOnDisk`)

## The change

A document keeps the file's modification time and length, stamped
before the read at open and after each save. A save that finds a
different stamp is refused once with `changed on disk since it was
opened: save again to overwrite, or reopen it`; the next save overwrites
on purpose. Rename and close-with-save get the same refusal (rename
rolls back; the tab stays open).

## Proof

Unit test `a_save_over_an_outside_change_is_refused_once_then_overwrites`
(our own saves pass; the outside change is kept on the first save).
Readback on the branch build: after the outside write, the first Cmd+S
left the disk as the other tool wrote it, with the message; the second
Cmd+S wrote backback's text.

## Not done

A clean buffer is not reloaded when the file changes on disk (VS Code
reloads it); the guard only stops a save over it.
