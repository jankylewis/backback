# Toolgap index

| Id | Gap | Status | For | Workaround | Path |
|---|---|---|---|---|---|
| BB-TG-001 | netcoredbg has no `runInTerminal`; real exit code unreadable | OBSERVED | none | Label exit codes "adapter-reported, unverified" | `D1/S01/BB_TG_001_NETCOREDBG_NO_RUN_IN_TERMINAL.md` |
| BB-TG-002 | No one-command gate; a background shell missed `cargo` | FIXED | none | `docs/release/scripts/gate.sh` (BB-TG-008) | `D1/S01/BB_TG_002_NO_ONE_COMMAND_GATE.md` |
| BB-TG-003 | gpui `closest_index_for_x` returns the line end past the last glyph | OBSERVED | BB-UIX-001 | Compare with the last glyph's middle in `Columns::column_at` | `D1/S01/BB_TG_003_GPUI_CLOSEST_INDEX_AT_LINE_END.md` |
| BB-TG-004 | The Apex CLI is not on nuget.org; CI skips 9 ApexTestkit tests | OBSERVED | BB-CIP-001 | Skip by name in CI; local gate runs them | `D1/S01/BB_TG_004_APEX_CLI_IS_NOT_PUBLIC.md` |
| BB-TG-005 | `screencapture -l` of a covered GPUI window is stale | OBSERVED | BB-UIX-002 | Prove with `state` steps + window list | `D1/S02/BB_TG_005_CAPTURE_OF_A_COVERED_WINDOW_IS_STALE.md` |
| BB-TG-006 | zsh `BG_NICE` nices an app started with `&` and all its children | OBSERVED | BB-PRC-003 | `open -a ... --args`, or `setopt NO_BG_NICE`; check `ps -o stat,nice` | `D1/S02/BB_TG_006_ZSH_BG_NICE_NICES_A_READBACK_LAUNCH.md` |
| BB-TG-007 | The agent shell is not Accessibility-trusted; posted clicks are dropped | OBSERVED | BB-BUG-004 | Window tests + `BB_KEYS`; a hand click by the headmate | `D1/S02/BB_TG_007_READBACK_SEAT_CANNOT_POST_INPUT.md` |
| BB-TG-008 | A hand-typed gate missed `BACKBACK_NETCOREDBG`; 14 debugger tests red | FIXED | none | `docs/release/scripts/gate.sh` sets PATH and the debugger path | `D1/S02/BB_TG_008_A_HAND_TYPED_GATE_MISSED_THE_DEBUGGER_PATH.md` |
| BB-TG-009 | A readback launch takes the keyboard focus; typed keys land in the app | OBSERVED | none | Writable runs on scratch copies only; real solutions `--no-write` | `D1/S03/BB_TG_009_A_READBACK_LAUNCH_TAKES_THE_KEYBOARD_FOCUS.md` |
| BB-TG-010 | A locked screen paints no frames; script mouse steps miss | OBSERVED | none | Check `CGSSessionScreenIsLocked`; window tests for mouse; rerun unlocked | `D1/S03/BB_TG_010_A_LOCKED_SCREEN_PAINTS_NO_FRAMES.md` |
| BB-TG-011 | A rebase can push a folder over its five-entry cap | OBSERVED | none | Audit (or `gate.sh`) after every rebase, before the push | `D1/S03/BB_TG_011_A_REBASE_CAN_PUSH_A_FOLDER_OVER_ITS_CAP.md` |
| BB-TG-012 | `open` from a terminal passes the shell environment; not a Dock launch | OBSERVED | BB-BUG-009 | `env -i` with launchd PATH for user-like launches | `D1/S03/BB_TG_012_OPEN_FROM_A_TERMINAL_IS_NOT_A_DOCK_LAUNCH.md` |
| BB-TG-013 | `bb_bundle.sh` without `cargo` on PATH ended in a JSON trace | FIXED | none | `bb_bundle.sh` adds `$HOME/.cargo/bin`, else refuses (exit 2) | `D2/S04/BB_TG_013_THE_BUNDLE_SCRIPT_MISSED_CARGO_OFF_PATH.md` |
| BB-TG-014 | A test binary from a shared `CARGO_TARGET_DIR` kept a removed worktree's `CARGO_MANIFEST_DIR` | OBSERVED | none | `touch` the worktree's `*.rs` before a gate in a shared target dir, or one target dir per worktree | `D2/S04/BB_TG_014_A_SHARED_TARGET_DIR_KEEPS_ANOTHER_WORKTREES_MANIFEST_PATH.md` |
| BB-TG-015 | A readback that asks no question reads intel state the server never got | FIXED | BB-BUG-019 | The runner flushes after each step (`flush_as_a_paint`); on older builds ask a question first | `D2/S04/BB_TG_015_A_READBACK_THAT_ASKS_NO_QUESTION_READS_STATE_THE_SERVER_NEVER_GOT.md` |
| BB-TG-016 | Readback steps waited for a file that never opened (binary file, folder launch) | FIXED | BB-BUG-029, BB-BUG-030 | A failed open and a folder launch start the steps; `open:<path>` step | `D2/S04/BB_TG_016_READBACK_STEPS_WAITED_FOR_A_FILE_THAT_NEVER_OPENED.md` |
| BB-TG-017 | A load test's count short of the total did not say which project failed | FIXED | gate 2026-11-04 | `wait_ready` returns and prints `Ready`'s failed projects | `D2/S05/BB_TG_017_A_LOAD_COUNT_SHORT_OF_THE_TOTAL_HID_ITS_CAUSE.md` |
| BB-TG-018 | Readback wall time per key grew 80 -> 490 ms over rounds while the main thread idled | OBSERVED | none | Judge by `bb_keys` UI work, not wall time; split long runs | `D2/S05/BB_TG_018_READBACK_WALL_TIME_STRETCHES_WHILE_THE_APP_IDLES.md` |
| BB-TG-019 | Runner presses miss UI a hidden window never drew | OBSERVED | BB-BUG-043 | Window tests for mouse; runner presses only with the window in front | `D2/S05/BB_TG_019_RUNNER_PRESSES_MISS_UI_A_HIDDEN_WINDOW_NEVER_DREW.md` |
| BB-TG-020 | Agents read whole files to find one item; no code map | FIXED | none | `bb map` (index, outline, `--find`), read from source each call | `D2/S05/BB_TG_020_AGENTS_READ_WHOLE_FILES_TO_FIND_ONE_ITEM.md` |
| BB-TG-021 | A sum of RSS overstated CI memory | FIXED | none | `vm_stat` in-use pages + swap in `ci_tests.sh` | `D2/S06/BB_TG_021_A_SUM_OF_RSS_OVERSTATED_CI_MEMORY.md` |
| BB-TG-022 | The audit failed the tool-fixed `.claude/skills` and `.agents/skills` layouts | FIXED | none | `AGENT_TREES` skips folder rules there; line caps stay | `D2/S06/BB_TG_022_THE_AUDIT_HAD_NO_RULE_FOR_AGENT_SKILL_TREES.md` |
