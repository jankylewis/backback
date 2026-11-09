# BB-BUG-035 — fix

id: BB-BUG-035
family: UIX
status: FIXED
source: `bb_crates/bb_core/bb_text/src/document/io/bb_disk.rs` (`reload_if_changed`), `bb_crates/shell/desktop/src/editor/mod.rs` (`watch_disk`, `reload_changed`)

## The change

Every 2 s each workspace window checks its open files (one `stat` each).
A tab whose file changed and that holds no unsaved edits takes the
disk's text: one undo step, the cursor kept where it can be, the edit
sent to the syntax tree and Roslyn like any other. A tab with unsaved
edits is never touched; its save meets BB-BUG-034's refusal.

## Proof

Unit test `a_clean_buffer_takes_an_outside_change_and_a_dirty_one_keeps_its_text`
(reload, clean after it, one undo back, a dirty buffer kept). Readback
on the branch build: an outside write at 3 s, then `x` and Cmd+S at 7 s:
the disk read `x` + the other tool's text, no refusal.
