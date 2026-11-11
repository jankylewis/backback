# Triage — read first

One row per failure reason and tool gap; grep the signature in your log.
Bugs a user met in the app have their own table: `../bugs/TRIAGE.md`.

| Id | Signature | Status | First check |
|---|---|---|---|
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
| BB-GAT-002 | `fixture build failed` ... `being used by another process` | FIXED | Do parallel test processes share a fixture folder? Use `OwnedTempDir` |
| BB-GAT-003 | `discovered=0 loading=1` in a native window test | FIXED | Does the test sleep a fixed time? Use `wait:tests` |
| BB-GAT-004 | a menu test counts one item too few | FIXED | Did a menu or key change skip the launch tests? Update the count and name the item |
| BB-GAT-005 | `the search did not end` in a window test | FIXED | Does the test wait for a state only a later step starts? Read it before that step |
| BB-GAT-006 | `clippy rc=101` on a tree whose tests passed | FIXED | Was the worktree edited while its gate ran? Rerun on the still tree |
| BB-DAP-001 | `CLR/System.InvalidOperationException: Exception thrown:` | FIXED | Is the headline read from `details`, not the top-level id and description? |
| BB-LSP-006 | `restored  : false after 120` | FIXED | Is `Restore complete` above the last `Successfully completed load` in the run log? |
| BB-ENV-001 | `PATH=/usr/bin:/bin:/usr/sbin:/sbin` in `ps eww` of the app | FIXED | Started by launchd? The login shell's PATH must be adopted and installed tools found in their own folders |
| BB-UIX-004 | `a lone \r in a saved CRLF file` | FIXED | Did a move or edit treat `\r` and `\n` as two stops? |
| BB-UIX-005 | `Option+Right moves one character` | FIXED | Does the key map read `modifiers.alt` and `modifiers.control` before the key name? |
| BB-UIX-006 | `the line ends under the pane's edge` | FIXED | Is the row's text in a clipped viewport shifted by `scroll_x`? |
| BB-UIX-007 | `a selected line vanishes on Tab` | FIXED | Is every editing key listed with what it does to a selection? |
| BB-UIX-008 | `a standard Cmd key does nothing` | FIXED | Has the Cmd table been swept against VS Code's default keymap? |
| BB-LSP-007 | `errors stay in another open file after an edit` | FIXED | Does the pull after a change or a refresh cover every open document? |
| BB-TG-015 | `bb_diag_state version=None` | FIXED | Is the binary older than the runner flush (`flush_as_a_paint`)? Then ask a question before reading `problems` |
| BB-UIX-009 | `a typed name glued onto the old one` | FIXED | Is the prefilled name selected when the box opens? |
| BB-UIX-010 | `a line-command key does nothing` | FIXED | Is the key bound in the app (`KeyBinding::new`) before the editor sees it? |
| BB-UIX-011 | `the current line's band ends at the text` | FIXED | Does the element that takes the click span its pane? |
| BB-UIX-012 | `a ▾ that does nothing` | FIXED | Is the marker a literal in a label? |
| BB-UIX-013 | `cannot find how to stop` | FIXED | Is the stop on the row that started the run? |
| BB-PRC-004 | `a <prefix>-<pid> folder of a dead pid` | FIXED | Is the folder made through `OwnedTempDir` or swept by its family? |
| BB-UIX-014 | `only __pycache__ in a package` | OBSERVED | The sources are gone from the checkout, not hidden |
| BB-TG-016 | `a readback hangs at 0 % CPU` | FIXED | Did the launch open a file? Binary and folder launches start the steps since BB-TG-016 |
| BB-PRF-003 | `roslyn-language-server RSS grows past 2 GB` | FIXED | Is `DOTNET_gcServer` set in the app's environment (a user value wins)? |
| BB-PRF-004 | `UI work grows with line length` | FIXED | Does the slow path go through `Columns::shape` or `styled_line` (both capped)? |
| BB-PRF-005 | `two roslyn processes per window` | FIXED | Does `bb_direct::server_behind` find exactly one server in the tool store? |
| BB-PRF-006 | `Find in Files slow on a big folder` | FIXED | `find_bench <root> <query>`: walk_ms vs median; are both phases used? |
| BB-TG-017 | `progress.last() (2, 3) != (3, 3)` | FIXED | Read the failed list in the message: which project did Roslyn fail to load? |
| BB-UIX-015 | `an outside change is lost after a save` | FIXED | Does `save` see `DiskGuard.seen` (stamped at `open`)? |
| BB-TG-018 | `a readback gets slower each round` | OBSERVED | Is `ui_work` still small? Then the runner's timers are throttled (likely App Nap), not the app |
| BB-UIX-016 | `tree rows differ from ls -A` | FIXED | Does `Explorer::refresh` run (editor `refresh_tree`)? |
| BB-PRC-005 | `a project load fails only with two workers` | FIXED | Do two owners in one process share a folder name? |
| BB-PRC-006 | `a temp folder kept although its owner is gone` | FIXED | Is the pid's process younger than the folder (`ps -o etime=`)? |
| BB-PRC-007 | `LEAK [ ... ] intel::bb_worker::tests` | OBSERVED | Which child holds the test's pipe: Roslyn, a dotnet build server? |
| BB-BLD-003 | `still broken after the fix merged` | FIXED | `BBCommit` of the running app (status bar) holds the fix? |
| BB-TG-019 | `a runner press outside a popup did nothing` | OBSERVED | Did the readback window draw a frame after the popup opened? |
| BB-TG-020 | `where is X defined / what does this crate do` | FIXED | Run `bb map` / `bb map --find X` before opening files |
| BB-TG-021 | `test memory peak above the runner's RAM` | FIXED | Is it a sum of RSS? Read `vm_stat` in-use pages and swap |
| BB-TG-022 | `FolderEntries` or `Marker` on `.claude/skills` or `.agents/skills` | FIXED | Is the walk skipping folder rules for the root agent trees? |
| BB-CIP-004 | `not acquired by Runner of type hosted` | FIXED | Rerun it; merge only when every `--json bucket` is `pass` |
| BB-CIP-005 | `OPEN` then the worktree is gone | FIXED | Is cleanup gated on the state value MERGED? |
| BB-ENV-002 | the machine runs hot during a gate | FIXED | Is `CARGO_BUILD_JOBS` 1 and `--test-threads=1`? Nothing else heavy running? |
| BB-TG-023 | a gate takes many minutes for a small change | FIXED | `gate.sh --plan`: is the scope right? `--full` only when unsure |
| BB-LSP-008 | `stopped within one second: Elapsed(())` in the dead-server test | OBSERVED | Is the runner loaded? Rerun it alone; time kill to `Stopped` |
