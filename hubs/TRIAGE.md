# Triage — read first

One row per bug and per failure reason. Grep the signature in your log.

| Id | Signature | Status | First check |
|---|---|---|---|
| BB-BUG-001 | `Cmd+M does nothing` | FIXED | Window menu has a Minimize item bound to `cmd-m` |
| BB-BUG-002 | `Explorer rows not in Menlo` | FIXED | Every surface reads one font token, not a literal |
| BB-BUG-003 | `HARNESS Select an MCP server` | FIXED | Build time of the running binary vs `master` (BB-BLD-001) |
| BB-BUG-004 | `Test Explorer click no response` | OBSERVED | Not reproduced by hand (2026-11-03); check the input reached the window (BB-TG-007) |
| BB-BUG-005 | `text after the cursor shifts right` | FIXED | Cursor is an overlay, never a char in the text runs (BB-UIX-001) |
| BB-BUG-007 | `end key does not move the cursor` | FIXED | Editor keys: only left/right handled in `input.rs` |
| BB-BUG-008 | `Debug failed: netcoredbg not found` | FIXED | Opened from the Dock? `PATH` is minimal; check `~/.local/share/backback/netcoredbg` |
| BB-BUG-009 | `1 project failed to load` only from the Dock | FIXED | Launched outside a terminal? `ps eww` the PATH |
| BB-BUG-010 | `a lone \r in a saved CRLF file` | FIXED | Did a cursor stop between `\r` and `\n`? |
| BB-BUG-011 | `Option+Backspace deletes a single char` | FIXED | Does the key map read `modifiers.alt`? |
| BB-BUG-012 | `Ctrl+A / Ctrl+E leave the cursor where it was` | FIXED | Does the key map read `modifiers.control`? |
| BB-BUG-006 | `EXPLORER / No solution` | FIXED | Finder launch has no argv; File > Open exists |
| BB-BLD-001 | `app built before the fix is still running` | FIXED | `ps -o lstart` and binary mtime vs merge time |
| BB-BLD-002 | `.app opens no window` | FIXED | `CFBundleExecutable` is the GUI binary; run `verify_bundle.sh` |
| BB-CIP-001 | `roslyn-language-server not found; set BACKBACK_ROSLYN_LS` | FIXED | `bb_ci.yml` installs the pinned Roslyn and netcoredbg |
| BB-CIP-003 | `error[yanked]: detected yanked crate` | FIXED | Ask crates.io for the version list; `cargo update -p <crate>` |
| BB-LSP-001 | `before ready: ` | FIXED | Wait on readiness, count projects by distinct path |
| BB-PRF-001 | `RSS 1,008 MB with the 20 MB file` | FIXED | Heap profile: who holds the bytes before changing code |
| BB-PRF-002 | `Go to File open freezes the window on a big folder` | FIXED | `BB_KEYS` UI work max at the feature cap (50,000 files) |
| BB-UIX-001 | `push_run(&mut runs, '│'` | FIXED | Every x on a line comes from `Columns` (one shaped line) |
| BB-LSP-002 | `trace.rs:38:5` | FIXED | Wait for Diagnostics with `first_diagnostics`, never `next()` |
| BB-CIP-002 | `gate:queued` | FIXED | `runs-on` is a label a hosted image serves (`macos-latest`) |
| BB-LSP-003 | `problems=18 after backspace` | OPEN | Poll `state` each second after an edit; check the shown version |
| BB-UIX-002 | `min window_on_screen=23` | FIXED | Window commands use the dispatching Window, not `active_window()` |
| BB-PRC-001 | `process left after quit` | FIXED | Record the app's process tree before quit; check every pid after |
| BB-LSP-004 | `open_with_restore_flag_lets_roslyn_restore` | FIXED | Does `bb open --restore` wait for the restore to finish? |
| BB-LSP-005 | `no server line of the pull after the load` | FIXED | Is the asserted log line written for our request, or by a side effect? |
| BB-UIX-003 | `gutter stayed 50 px at zoom` | FIXED | Every width that holds text scales with `Typography` |
| BB-PRC-002 | `Building Harness.slnx... (never ends)` | FIXED | `ps -o stat` of the child: `SN` = spawned from a background thread |
| BB-PRC-003 | `SN roslyn-language-server` | OBSERVED | Is the app itself `N`? A zsh `&` launch is niced (BB-TG-006) |
| BB-GAT-001 | `incremental_parse_equals_a_fresh_parse ... FAILED` | FIXED | Is the text broken? Then only the extent must match |
| BB-DAP-001 | `CLR/System.InvalidOperationException: Exception thrown:` | FIXED | Is the headline read from `details`, not the top-level id and description? |
| BB-LSP-006 | `restored  : false after 120` | FIXED | Is `Restore complete` above the last `Successfully completed load` in the run log? |
| BB-ENV-001 | `PATH=/usr/bin:/bin:/usr/sbin:/sbin` in `ps eww` of the app | FIXED | Started by launchd? The login shell's PATH must be adopted and installed tools found in their own folders |
| BB-UIX-004 | `a lone \r in a saved CRLF file` | FIXED | Did a move or edit treat `\r` and `\n` as two stops? |
| BB-UIX-005 | `Option+Right moves one character` | FIXED | Does the key map read `modifiers.alt` and `modifiers.control` before the key name? |
| BB-BUG-013 | `cannot horizontally scroll` | FIXED | Does the tab keep a `scroll_x` and does `state` print it after End on a wide line? |
| BB-UIX-006 | `the line ends under the pane's edge` | FIXED | Is the row's text in a clipped viewport shifted by `scroll_x`? |
| BB-BUG-014 | `Cmd+Backspace does nothing` | FIXED | Does `command_key` map `backspace` and `delete` to the line-edge deletes? |
| BB-BUG-015 | `Tab with a selection replaces it` | FIXED | Does `text_key` give `tab` its own arm before the insert default? |
| BB-UIX-007 | `a selected line vanishes on Tab` | FIXED | Is every editing key listed with what it does to a selection? |
| BB-BUG-016 | `Cmd+A does nothing` | FIXED | Does `command_key` have an `"a"` arm to `select_all`? |
| BB-BUG-017 | `Cmd+/ does nothing` | FIXED | Does `command_key` have a `"/"` arm to `toggle_line_comment`? |
| BB-UIX-008 | `a standard Cmd key does nothing` | FIXED | Has the Cmd table been swept against VS Code's default keymap? |
| BB-BUG-018 | `HalfHalve` | FIXED | Does `RenameInput` open `fresh` and does the first key clear it? |
| BB-BUG-019 | `problems=11 after a rename` | FIXED | After a change, does the server thread mark every open document, not only the edited one? |
| BB-LSP-007 | `errors stay in another open file after an edit` | FIXED | Does the pull after a change or a refresh cover every open document? |
| BB-TG-015 | `bb_diag_state version=None` | FIXED | Is the binary older than the runner flush (`flush_as_a_paint`)? Then ask a question before reading `problems` |
| BB-UIX-009 | `a typed name glued onto the old one` | FIXED | Is the prefilled name selected when the box opens? |
| BB-BUG-020 | `Cmd+Shift+K does nothing` | FIXED | Does `document_command` have arms for `k` and `enter`? |
| BB-BUG-021 | `Alt+Down does nothing` | FIXED | Is `move_lines` reached (`text_key`, Alt arm before the plain arrows)? Cmd+[ / Cmd+] are Back/Forward |
| BB-UIX-010 | `a line-command key does nothing` | FIXED | Is the key bound in the app (`KeyBinding::new`) before the editor sees it? |
| BB-BUG-022 | `a click past the text does nothing` | FIXED | Does the editor row span the pane (`w_full` in `row_frame`)? |
| BB-BUG-023 | `which sidebar side is chosen?` | FIXED | Do both headers draw `side_tabs`? |
| BB-UIX-011 | `the current line's band ends at the text` | FIXED | Does the element that takes the click span its pane? |
| BB-BUG-024 | `a test group does not fold` | FIXED | Is the head a `group_row` with a `Group` key? |
| BB-BUG-025 | `a click on a test does not open it` | FIXED | Does `find_source` find the class under the project folder? |
| BB-BUG-026 | `no way to stop a test` | FIXED | Does the running row show ■, and does the process group die? |
| BB-UIX-012 | `a ▾ that does nothing` | FIXED | Is the marker a literal in a label? |
| BB-UIX-013 | `cannot find how to stop` | FIXED | Is the stop on the row that started the run? |
| BB-BUG-027 | `$TMPDIR keeps growing` | FIXED | Run `bb sweep`; which prefix is left with a live pid? |
| BB-PRC-004 | `a <prefix>-<pid> folder of a dead pid` | FIXED | Is the folder made through `OwnedTempDir` or swept by its family? |
| BB-BUG-028 | `Cmd+C does not copy` | FIXED | Does `clip::clipboard_key` see the key before `command_key`? |
| BB-BUG-029 | `the Explorer does not show the open file` | FIXED | Does `follow_active` run (render, or the runner's flush)? |
| BB-BUG-030 | `file is not UTF-8` | FIXED | A binary file: the message is by design |
| BB-BUG-031 | `the Explorer hides my files` | OBSERVED | `ls -A` the folder: is the file on disk? |
| BB-UIX-014 | `only __pycache__ in a package` | OBSERVED | The sources are gone from the checkout, not hidden |
| BB-TG-016 | `a readback hangs at 0 % CPU` | FIXED | Did the launch open a file? Binary and folder launches start the steps since BB-TG-016 |
| BB-PRF-003 | `roslyn-language-server RSS grows past 2 GB` | FIXED | Is `DOTNET_gcServer` set in the app's environment (a user value wins)? |
| BB-BUG-032 | `the editor freezes on a minified file` | FIXED | Is the line past `RENDER_CAP`? It should end in `…` |
| BB-BUG-033 | `the cursor stops inside é or an emoji` | FIXED | Does the key go through `graphemes::before` / `after`? |
| BB-PRF-004 | `UI work grows with line length` | FIXED | Does the slow path go through `Columns::shape` or `styled_line` (both capped)? |
| BB-TG-017 | `progress.last() (2, 3) != (3, 3)` | FIXED | Read the failed list in the message: which project did Roslyn fail to load? |
| BB-BUG-034 | `changed on disk since it was opened` | FIXED | By design: save again to overwrite, or reopen to take the disk's text |
| BB-UIX-015 | `an outside change is lost after a save` | FIXED | Does `save` see `DiskGuard.seen` (stamped at `open`)? |
| BB-BUG-035 | `the tab shows old text after git checkout` | FIXED | Does the tab hold unsaved edits? Those are never reloaded |
| BB-BUG-036 | `Cmd+P does nothing` | FIXED | Is `cmd-p` in `open::bindings`? Cmd+Shift+O vs Go to Symbol awaits a ruling |
| BB-TG-018 | `a readback gets slower each round` | OBSERVED | Is `ui_work` still small? Then the runner's timers are throttled (likely App Nap), not the app |
| BB-BUG-037 | `a new file is not in the Explorer` | FIXED | Wait 2 s (the disk check); is the folder loaded? |
| BB-UIX-016 | `tree rows differ from ls -A` | FIXED | Does `Explorer::refresh` run (editor `refresh_tree`)? |
| BB-BUG-038 | `(2, 3) != (3, 3); failed: [...]` | FIXED | Is each worker's folder `backback-intel-<pid>-<n>`? |
| BB-PRC-005 | `a project load fails only with two workers` | FIXED | Do two owners in one process share a folder name? |
| BB-PRC-006 | `a temp folder kept although its owner is gone` | FIXED | Is the pid's process younger than the folder (`ps -o etime=`)? |
| BB-BLD-003 | `still broken after the fix merged` | FIXED | `BBCommit` of the running app (status bar) holds the fix? |
| BB-TG-019 | `a runner press outside a popup did nothing` | OBSERVED | Did the readback window draw a frame after the popup opened? |
| BB-TG-020 | `where is X defined / what does this crate do` | FIXED | Run `bb map` / `bb map --find X` before opening files |
| BB-BUG-055 | `the cursor moved off screen` | FIXED | Does `after_key` call `scroll_to_item` (Nearest)? |
