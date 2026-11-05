# BB-BUG-050 — fix

id: BB-BUG-050
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/bb_explorer/files/` (`bb_ops.rs`, `mod.rs`, `trash.rs`, `view.rs`), `bb_rows/bb_row_view.rs` (right-click), `bb_quick_open/bb_model.rs` (`name_box`)

## The change

- A right-click on a row opens a menu at the pointer: New File..., New
  Folder... (in the folder, or the file's folder), Rename..., Delete. The
  root row offers only the two News.
- A name is typed in the Go to File box (a rename starts with the name);
  names are checked (not empty, no `/`, not `.`/`..`) and nothing is
  overwritten. A new file opens; the tree refreshes and selects the entry.
- Delete asks first (macOS prompt), then runs `/usr/bin/trash` off the UI
  thread with a 15 s deadline, so Finder's Put Back works.
- Open tabs follow a rename and close on a delete; a file with unsaved
  changes is refused ("Save or close ... first"). `--no-write` refuses all.
- The row files moved to `bb_explorer/bb_rows/` to keep five entries a folder.

## Lesson

A destructive file action goes to the Trash, never `remove_file`, and its
unit tests stand a seam in (`TRASH` is `/bin/rm` under `cfg(test)`) so a
test run never fills the user's Trash.

## Proof

`files::tests`: on-disk checks (no overwrite, bad names, case-only rename,
a refused Trash), and window tests for the menu, New File, Rename, Delete
(prompt answered), the unsaved-changes refusal and `--no-write`.
