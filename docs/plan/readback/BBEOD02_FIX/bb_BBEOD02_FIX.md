# BBEOD02-FIX readback (2026-11-01, backback_1)

Issues came from the independent verification of `fba483d` (B1–B3 + low items).
My pane has no Accessibility trust, so keys go through GPUI's own dispatch
(`BB_KEYS`, `window.dispatch_keystroke`) after a launch with a file argument.
OS-level input (mouse, menu clicks) is left for backback_2 to re-verify.

| case | steps | observed after fix | shot |
| --- | --- | --- | --- |
| B1 read-only | 0444 file, `BB_KEYS="x cmd-s"` | status bar "file is read-only"; sha, inode, mode unchanged; tab stays dirty | `B1_READONLY_SAVE.png` |
| B1 hard link | A.cs + link B.cs, `x cmd-s cmd-q` | B.cs shows the edit; inode same; nlink 2; app quit exit 0 | log only |
| B2 20 MB open | launch with Big.cs | first frame 156 ms; "Loading Big.cs…" tab; load 2.7 s on a worker | `B2_LOADING.png` |
| B2 typing | `i n t space` | text and highlight correct; UI work p99 2.8 ms | log only |
| B3 10k folder | launch on `many/` | first frame 197 ms, RSS 115 MB; f1, f2 … f10 order | `B3_MANY_10K.png` |
| open error | launch with Latin-1 file | status bar "file is not UTF-8" | log only |
| Quit dirty | `x cmd-q` | save prompt; app keeps running | `QUIT_DIRTY_PROMPT.png` |
| Quit clean | `cmd-q` | app exits, code 0 | log only |

Failing-first tests (each failed on `fba483d` code, passes now):
`read_only_file_save_fails_typed_and_keeps_bytes_and_inode`,
`hard_link_keeps_inode_and_both_names_see_the_save`,
`mode_and_extended_attribute_survive_save`,
`symlink_path_save_writes_target_and_keeps_link`,
`dangling_symlink_save_creates_target_and_keeps_link` (mutant-checked),
`files_sort_by_number_value_not_by_text`,
`every_window_error_reaches_the_status_bar_not_only_stderr` (9 sites at HEAD).
B2 and B3 are performance defects: their failing check is the bench in
`docs/plan/bench/BBEOD02/FIX.md` (before 244 ms p99 per key, 439 ms frame).

## Families and guards

- Save loses file identity (rename over target): read-only, hard link, owner,
  mode, xattrs, symlink, dangling symlink. Guard: one `save` path checks the
  target is writable, then writes in place or onto a metadata copy.
- Full-text work per keystroke: copy, compare, char scan, line table, enter's
  prefix copy, full reparse. Guard: every text change goes through one
  `splice` that logs a byte-level `TextChange`; dirty is a state id; parsing
  runs off the UI thread. Proptests: the change log replays to the new text,
  incremental parse equals a fresh parse, edits during a background parse.
- Two line-break definitions: ropey counts lone CR, NEL, LS, PS; tree-sitter
  counts only LF. Found by the change-log proptest. Guard: LF rows with a
  fast path only when no other break exists; load scan = edit scan proptest.
- Render cost grows with row count: Explorer now uses `uniform_list` like the
  editor body.
- Errors only on stderr: one `report` sets the status bar; a source guard test.

## Open

- Explorer refresh on disk change (FS events) not done: it needs a watcher
  crate and a new thread; recorded for a later BBEOD.
- RSS 1.0 GB with the 20 MB file open and edited; no before value.
- Menu click on "Quit Backback" not exercised (needs OS input); the action,
  binding and menu entry are tested.
