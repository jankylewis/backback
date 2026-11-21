# Triage — APP failures (UIX, LSP, DAP, TST)

One row per failure reason of this layer, by family and id; the entry point is
`hubs/failures/TRIAGE.md`. Grep the signature in your log.

| Id | Signature | Status | First check |
|---|---|---|---|
| BB-DAP-001 | `CLR/System.InvalidOperationException: Exception thrown:` | FIXED | Is the headline read from `details`, not the top-level id and description? |
| BB-LSP-001 | `before ready: ` | FIXED | Wait on readiness, count projects by distinct path |
| BB-LSP-002 | `trace.rs:38:5` | FIXED | Wait for Diagnostics with `first_diagnostics`, never `next()` |
| BB-LSP-003 | `problems=18 after backspace` | OPEN | Not reproduced on master (4/4, BBEOD35); did the keys reach the text? Run `roslyn_problems_return_...` |
| BB-LSP-004 | `open_with_restore_flag_lets_roslyn_restore` | FIXED | Does `bb open --restore` wait for the restore to finish? |
| BB-LSP-005 | `no server line of the pull after the load` | FIXED | Is the asserted log line written for our request, or by a side effect? |
| BB-LSP-006 | `restored  : false after 120` | FIXED | Is `Restore complete` above the last `Successfully completed load` in the run log? |
| BB-LSP-007 | `errors stay in another open file after an edit` | FIXED | Does the pull after a change or a refresh cover every open document? |
| BB-LSP-008 | `stopped within one second: Elapsed(())` in the dead-server test | FIXED | Does the clock time the kill too? `kill_child` is a crash-like KILL |
| BB-LSP-009 | `-32801` / `content modified` | FIXED | Does every request go through `Session::raw`? |
| BB-LSP-010 | `C# server: ` for a Rust or Python request | FIXED | Is the answering server passed to `answers::apply`? |
| BB-LSP-011 | `Renamed: 0 edits in 0 files` | FIXED | Was the plan empty (server still indexing)? |
| BB-LSP-012 | a new `Cargo.lock` after opening a Cargo folder | ACCEPTED | Accepted by ruling 2026-11-19 (standard for rust-analyzer clients); any other new file is a bug |
| BB-LSP-013 | `document is already open` after Restart Server | FIXED | Does `resync_open` get only the restarted server? Count `didOpen`s per start on the BBEOD39 seam |
| BB-UIX-001 | `push_run(&mut runs, '│'` | FIXED | Every x on a line comes from `Columns` (one shaped line) |
| BB-UIX-002 | `min window_on_screen=23` | FIXED | Window commands use the dispatching Window, not `active_window()` |
| BB-UIX-003 | `gutter stayed 50 px at zoom` | FIXED | Every width that holds text scales with `Typography` |
| BB-UIX-004 | `a lone \r in a saved CRLF file` | FIXED | Did a move or edit treat `\r` and `\n` as two stops? |
| BB-UIX-005 | `Option+Right moves one character` | FIXED | Does the key map read `modifiers.alt` and `modifiers.control` before the key name? |
| BB-UIX-006 | `the line ends under the pane's edge` | FIXED | Is the row's text in a clipped viewport shifted by `scroll_x`? |
| BB-UIX-007 | `a selected line vanishes on Tab` | FIXED | Is every editing key listed with what it does to a selection? |
| BB-UIX-008 | `a standard Cmd key does nothing` | FIXED | Has the Cmd table been swept against VS Code's default keymap? |
| BB-UIX-009 | `a typed name glued onto the old one` | FIXED | Is the prefilled name selected when the box opens? |
| BB-UIX-010 | `a line-command key does nothing` | FIXED | Is the key bound in the app (`KeyBinding::new`) before the editor sees it? |
| BB-UIX-011 | `the current line's band ends at the text` | FIXED | Does the element that takes the click span its pane? |
| BB-UIX-012 | `a ▾ that does nothing` | FIXED | Is the marker a literal in a label? |
| BB-UIX-013 | `cannot find how to stop` | FIXED | Is the stop on the row that started the run? |
| BB-UIX-014 | `only __pycache__ in a package` | OBSERVED | The sources are gone from the checkout, not hidden |
| BB-UIX-015 | `an outside change is lost after a save` | FIXED | Does `save` see `DiskGuard.seen` (stamped at `open`)? |
| BB-UIX-016 | `tree rows differ from ls -A` | FIXED | Does `Explorer::refresh` run (editor `refresh_tree`)? |
| BB-UIX-017 | ← is live, a click does nothing | FIXED | Do the arrow and the press use the same `lives` rule? Does `go` need `here`? |
| BB-UIX-018 | `file path: No such file or directory` with no file name | FIXED | Does the error arm call `open_failed` (drops `intel.jump`, `tests.pending_jump`)? |
| BB-UIX-019 | a deleted file's path in a start's breakpoints | FIXED | Is every session read through `for_session`? |
| BB-UIX-020 | `not on this machine` for a path under the root | FIXED | Does `jump_to` compare the mapped path with `view.root`? |
| BB-UIX-021 | window title = the file's folder; no server for a `.rs`/`.py` file | FIXED | Does `split_argument` ask `project_root`? Is the root inside the walk? |
| BB-UIX-022 | `C#` mark on a `.py`/`.rs` tab; C# or Debug menu without C# | FIXED | Does the chrome ask `Langs` / `Lang::of_source`? |
| BB-UIX-023 | RECENT above LANGUAGES; Down leaves the row off screen | FIXED | Is RECENT its own element after the ticks? Does `step` scroll? |
| BB-UIX-024 | `cargo metadata exited with Some(101): error:` over many lines | FIXED | Does the panel show `summary()` and keep `to_string()` for the hover? |
| BB-UIX-025 | `C# server:` with no C#; no `Project ·` for a crate; `Colours(CSharp)` on a `.rs`/new tab | FIXED | `grep -n "None =>\|unwrap_or(Lang::CSharp)\|csproj\|CSHARP"` on the path; does it use `server_of`/`of_project`/`file_mode`? |
| BB-UIX-026 | a `langs` record the user never picked; Python ticked in a C# window | FIXED | Is the store v2 (`picked`)? Does the window's ticks come from a pick or `default_ticks`? |
