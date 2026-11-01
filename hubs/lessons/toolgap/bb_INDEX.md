# Toolgap index

| Id | Gap | Status | For | Workaround | Path |
|---|---|---|---|---|---|
| BB-TG-001 | netcoredbg has no `runInTerminal`; real exit code unreadable | OBSERVED | none | Label exit codes "adapter-reported, unverified" | `S01/BB_TG_001_NETCOREDBG_NO_RUN_IN_TERMINAL.md` |
| BB-TG-002 | No one-command gate; a background shell missed `cargo` | OBSERVED | none | Export `$HOME/.cargo/bin` on PATH; check each step's exit code | `S01/BB_TG_002_NO_ONE_COMMAND_GATE.md` |
| BB-TG-003 | gpui `closest_index_for_x` returns the line end past the last glyph | OBSERVED | BB-UIX-001 | Compare with the last glyph's middle in `Columns::column_at` | `S01/BB_TG_003_GPUI_CLOSEST_INDEX_AT_LINE_END.md` |
| BB-TG-004 | The Apex CLI is not on nuget.org; CI skips 9 ApexTestkit tests | OBSERVED | BB-CIP-001 | Skip by name in CI; local gate runs them | `S01/BB_TG_004_APEX_CLI_IS_NOT_PUBLIC.md` |
| BB-TG-005 | `screencapture -l` of a covered GPUI window is stale | OBSERVED | BB-UIX-002 | Prove with `state` steps + window list | `S02/BB_TG_005_CAPTURE_OF_A_COVERED_WINDOW_IS_STALE.md` |
| BB-TG-006 | zsh `BG_NICE` nices an app started with `&` and all its children | OBSERVED | BB-PRC-003 | `open -a ... --args`, or `setopt NO_BG_NICE`; check `ps -o stat,nice` | `S02/BB_TG_006_ZSH_BG_NICE_NICES_A_READBACK_LAUNCH.md` |
| BB-TG-007 | The agent shell is not Accessibility-trusted; posted clicks are dropped | OBSERVED | BB-BUG-004 | Window tests + `BB_KEYS`; a hand click by the headmate | `S02/BB_TG_007_READBACK_SEAT_CANNOT_POST_INPUT.md` |
