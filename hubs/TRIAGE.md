# Triage — read first

One row per bug and per failure reason. Grep the signature in your log.

| Id | Signature | Status | First check |
|---|---|---|---|
| BB-BUG-001 | `Cmd+M does nothing` | FIXED | Window menu has a Minimize item bound to `cmd-m` |
| BB-BUG-002 | `Explorer rows not in Menlo` | FIXED | Every surface reads one font token, not a literal |
| BB-BUG-003 | `HARNESS Select an MCP server` | FIXED | Build time of the running binary vs `master` (BB-BLD-001) |
| BB-BUG-004 | `Test Explorer click no response` | OPEN | Reproduce by hand before blaming the input driver |
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
| BB-BUG-021 | `Alt+Down does nothing` | OPEN | Move and copy are not in the text model (Cmd+L is: `select_line`); Cmd+[ / Cmd+] are Back/Forward |
| BB-UIX-010 | `a line-command key does nothing` | OPEN | Is the key bound in the app (`KeyBinding::new`) before the editor sees it? |
