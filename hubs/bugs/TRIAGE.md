# Triage — bugs

One row per bug (`BB-BUG-*`); failure reasons and tool gaps are in
`../TRIAGE.md`. Grep the signature in your log.

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
| BB-BUG-013 | `cannot horizontally scroll` | FIXED | Does the tab keep a `scroll_x` and does `state` print it after End on a wide line? |
| BB-BUG-014 | `Cmd+Backspace does nothing` | FIXED | Does `command_key` map `backspace` and `delete` to the line-edge deletes? |
| BB-BUG-015 | `Tab with a selection replaces it` | FIXED | Does `text_key` give `tab` its own arm before the insert default? |
| BB-BUG-016 | `Cmd+A does nothing` | FIXED | Does `command_key` have an `"a"` arm to `select_all`? |
| BB-BUG-017 | `Cmd+/ does nothing` | FIXED | Does `command_key` have a `"/"` arm to `toggle_line_comment`? |
| BB-BUG-018 | `HalfHalve` | FIXED | Does `RenameInput` open `fresh` and does the first key clear it? |
| BB-BUG-019 | `problems=11 after a rename` | FIXED | After a change, does the server thread mark every open document, not only the edited one? |
| BB-BUG-020 | `Cmd+Shift+K does nothing` | FIXED | Does `document_command` have arms for `k` and `enter`? |
| BB-BUG-021 | `Alt+Down does nothing` | FIXED | Is `move_lines` reached (`text_key`, Alt arm before the plain arrows)? Cmd+[ / Cmd+] are Back/Forward |
| BB-BUG-022 | `a click past the text does nothing` | FIXED | Does the editor row span the pane (`w_full` in `row_frame`)? |
| BB-BUG-023 | `which sidebar side is chosen?` | FIXED | Do both headers draw `side_tabs`? |
| BB-BUG-024 | `a test group does not fold` | FIXED | Is the head a `group_row` with a `Group` key? |
| BB-BUG-025 | `a click on a test does not open it` | FIXED | Does `find_source` find the class under the project folder? |
| BB-BUG-026 | `no way to stop a test` | FIXED | Does the running row show ■, and does the process group die? |
| BB-BUG-027 | `$TMPDIR keeps growing` | FIXED | Run `bb sweep`; which prefix is left with a live pid? |
| BB-BUG-028 | `Cmd+C does not copy` | FIXED | Does `clip::clipboard_key` see the key before `command_key`? |
| BB-BUG-029 | `the Explorer does not show the open file` | FIXED | Does `follow_active` run (render, or the runner's flush)? |
| BB-BUG-030 | `file is not UTF-8` | FIXED | A binary file: the message is by design |
| BB-BUG-031 | `the Explorer hides my files` | OBSERVED | `ls -A` the folder: is the file on disk? |
| BB-BUG-032 | `the editor freezes on a minified file` | FIXED | Is the line past `RENDER_CAP`? It should end in `…` |
| BB-BUG-033 | `the cursor stops inside é or an emoji` | FIXED | Does the key go through `graphemes::before` / `after`? |
| BB-BUG-034 | `changed on disk since it was opened` | FIXED | By design: save again to overwrite, or reopen to take the disk's text |
| BB-BUG-035 | `the tab shows old text after git checkout` | FIXED | Does the tab hold unsaved edits? Those are never reloaded |
| BB-BUG-036 | `Cmd+P does nothing` | FIXED | Is `cmd-p` in `open::bindings`? Cmd+Shift+O vs Go to Symbol awaits a ruling |
| BB-BUG-037 | `a new file is not in the Explorer` | FIXED | Wait 2 s (the disk check); is the folder loaded? |
| BB-BUG-038 | `(2, 3) != (3, 3); failed: [...]` | FIXED | Is each worker's folder `backback-intel-<pid>-<n>`? |
| BB-BUG-055 | `the cursor moved off screen` | FIXED | Does `after_key` call `scroll_to_item` (Nearest)? |
| BB-BUG-056 | `the line end stays under the Explorer` | FIXED | Is the tool window docked right? Does the width come from `editor_area`? |
| BB-BUG-057 | `cannot paste in Find in Files` | FIXED | Does `key` pass Cmd+V's text to `QuickOpen::press`? |
| BB-BUG-058 | `cannot scroll up the terminal` | FIXED | Does the grid's `on_scroll` reach `Session::scroll`? |
| BB-BUG-059 | `only 3 recent workspaces show` | FIXED | Is the saved list full of deleted folders? `recent::add` drops them |
