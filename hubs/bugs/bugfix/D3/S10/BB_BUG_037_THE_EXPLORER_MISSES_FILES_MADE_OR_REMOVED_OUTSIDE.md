# BB-BUG-037 — fix

id: BB-BUG-037
family: UIX
status: FIXED
source: `bb_crates/bb_core/explorer/src/tree/children.rs` (`refresh_entry`), `bb_crates/bb_core/explorer/src/tree/bb_model.rs` (`Explorer::refresh`), `bb_crates/shell/desktop/src/editor/mod.rs` (`refresh_tree`)

## The change

The 2 s disk check (BB-BUG-035) also re-reads every loaded folder and
merges by path: a kept entry keeps its open state and children, a new
one comes in closed, a gone one goes; the selection stays on its path.

## Proof

Unit test `refresh_takes_outside_changes_and_keeps_open_folders_open`.
Readback on the branch build: after an outside create and delete,
`open:new.py` selected `new.py` in the tree (before the fix it stayed on
`sub/b.txt`).

## Still open

The re-read runs on the UI thread: one `read_dir` per loaded folder
every 2 s. Fine for hundreds of folders; a tree with thousands loaded
would want it off-thread or on file events.
