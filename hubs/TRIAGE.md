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
| BB-BUG-006 | `EXPLORER / No solution` | FIXED | Finder launch has no argv; File > Open exists |
| BB-BLD-001 | `app built before the fix is still running` | FIXED | `ps -o lstart` and binary mtime vs merge time |
| BB-BLD-002 | `.app opens no window` | FIXED | `CFBundleExecutable` is the GUI binary; run `verify_bundle.sh` |
| BB-CIP-001 | `roslyn-language-server not found; set BACKBACK_ROSLYN_LS` | FIXED | `bb_ci.yml` installs the pinned Roslyn and netcoredbg |
| BB-LSP-001 | `before ready: ` | FIXED | Wait on readiness, count projects by distinct path |
| BB-PRF-001 | `RSS 1,008 MB with the 20 MB file` | FIXED | Heap profile: who holds the bytes before changing code |
| BB-UIX-001 | `push_run(&mut runs, '│'` | FIXED | Every x on a line comes from `Columns` (one shaped line) |
| BB-LSP-002 | `trace.rs:38:5` | FIXED | Wait for Diagnostics with `first_diagnostics`, never `next()` |
| BB-CIP-002 | `gate:queued` | FIXED | `runs-on` is a label a hosted image serves (`macos-latest`) |
| BB-LSP-003 | `problems=18 after backspace` | OPEN | Poll `state` each second after an edit; check the shown version |
| BB-UIX-002 | `min window_on_screen=23` | FIXED | Window commands use the dispatching Window, not `active_window()` |
| BB-PRC-001 | `process left after quit` | FIXED | Record the app's process tree before quit; check every pid after |
| BB-LSP-004 | `open_with_restore_flag_lets_roslyn_restore` | FIXED | Does `bb open --restore` wait for the restore to finish? |
| BB-UIX-003 | `gutter stayed 50 px at zoom` | FIXED | Every width that holds text scales with `Typography` |
| BB-PRC-002 | `Building Harness.slnx... (never ends)` | FIXED | `ps -o stat` of the child: `SN` = spawned from a background thread |
| BB-PRC-003 | `SN roslyn-language-server` | OBSERVED | Is the app itself `N`? A zsh `&` launch is niced (BB-TG-006) |
