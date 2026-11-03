# Bugfix index

| Id | Fix | Status | Proof | Path |
|---|---|---|---|---|
| BB-BUG-003 | Panel removed in PR #9; running a fresh build shows no panel | FIXED | guard tests + no-MCP screenshot in #9 | `D1/S01/BB_BUG_003_MCP_PANEL_STILL_SHOWN.md` |
| BB-BUG-005 | Caret is a quad over one shaped line; clicks and popups measure the same line | FIXED | 4 red->green window tests + pixel-diff readback | `D1/S02/BB_BUG_005_TEXT_SHIFTS_WHEN_THE_CURSOR_MOVES.md` |
| BB-BUG-001 | Window menu + Cmd+M/Cmd+H/Cmd+Alt+H | FIXED | 2 red-first tests + readback (window minimized) | `D1/S01/BB_BUG_001_CMD_M_DOES_NOT_MINIMIZE.md` |
| BB-BUG-007 | Home/End smart, Cmd+arrows, Shift extends | FIXED | 5 red-first tests + readback cursor log | `D1/S02/BB_BUG_007_HOME_AND_END_DO_NOTHING.md` |
| BB-BUG-006 | Finder launch opens the recent workspace; Cmd+O; recent file | FIXED | red-first resolve test + readback | `D1/S02/BB_BUG_006_APP_OPENS_THE_DISK_ROOT.md` |
| BB-BUG-002 | One Typography, set only at the window root; Settings, zoom | FIXED | source scan + window tests + live capture | `D1/S01/BB_BUG_002_NOT_ONE_FONT_EVERYWHERE.md` |
| BB-BUG-008 | `locate` also checks ~/.local/share/backback/netcoredbg and Homebrew | FIXED | red-first locate test + installed-app readback | `D1/S02/BB_BUG_008_F5_FROM_THE_DOCK_FINDS_NO_NETCOREDBG.md` |
| BB-BUG-009 | Adopt the login shell PATH at start (VS Code style); .NET folders as fallback | FIXED | 4 tests + launchd-env readback | `D1/S03/BB_BUG_009_AN_APP_FROM_THE_DOCK_FINDS_NO_DOTNET.md` |
| BB-BUG-010 | `\r\n` is one break: arrows step over it, Backspace/Delete remove it whole | FIXED | 3 red-first tests + byte readback | `D1/S03/BB_BUG_010_RIGHT_AT_A_LINE_END_SPLITS_A_CRLF.md` |
| BB-BUG-011 | Option+Left/Right/Backspace/Delete work by word (VS Code / IntelliJ stops) | FIXED | 6 red-first tests + window test | `D1/S03/BB_BUG_011_OPTION_ARROWS_MOVE_ONE_CHAR_NOT_ONE_WORD.md` |
| BB-BUG-012 | Ctrl+A/E/B/F/P/N/H/D work as in Cocoa and VS Code on macOS | FIXED | red-first window test | `D1/S03/BB_BUG_012_THE_MACOS_CTRL_TEXT_KEYS_DO_NOTHING.md` |
| BB-BUG-013 | A per-tab `scroll_x`: clipped text viewport per row, caret and clicks offset, wheel scrolls, the caret kept in sight on moves | FIXED | red-first window test + readback on the headmate's file | `D2/S04/BB_BUG_013_LONG_LINES_ARE_CLIPPED_WITH_NO_HORIZONTAL_SCROLL.md` |
| BB-BUG-014 | `delete_to_line_start` / `delete_to_line_end` on Cmd+Backspace / Cmd+Delete | FIXED | red-first unit + window tests | `D2/S04/BB_BUG_014_CMD_BACKSPACE_DOES_NOTHING.md` |
| BB-BUG-015 | Tab indents and Shift+Tab outdents the selected lines, the selection kept, one undo step | FIXED | red-first unit + window tests | `D2/S04/BB_BUG_015_TAB_WITH_A_SELECTION_REPLACES_IT.md` |
| BB-BUG-016 | `select_all` on Cmd+A | FIXED | red-first unit + window test | `D2/S04/BB_BUG_016_CMD_A_DOES_NOT_SELECT_ALL.md` |
| BB-BUG-017 | `toggle_line_comment` on Cmd+/: `// ` after the indent, blank lines skipped, one undo step | FIXED | red-first unit + window test | `D2/S05/BB_BUG_017_CMD_SLASH_DOES_NOT_TOGGLE_A_LINE_COMMENT.md` |
| BB-BUG-018 | The old name opens selected; the first key replaces it | FIXED | red-first window test + sweep 3 readback | `D2/S05/BB_BUG_018_THE_RENAME_BOX_APPENDS_TO_THE_OLD_NAME.md` |
| BB-BUG-019 | Every change re-pulls every open file; the refresh request is declared and followed | FIXED | red-first server-thread test + readback | `D2/S05/BB_BUG_019_PROBLEMS_KEEP_STALE_DIAGNOSTICS_AFTER_A_RENAME.md` |
| BB-BUG-020 | `delete_lines`, `insert_line_below`/`above` in the text model; Cmd+Shift+K and Cmd+(Shift+)Enter arms | FIXED | red-first unit + window tests + sweep 4 readback | `D2/S05/BB_BUG_020_CMD_SHIFT_K_AND_CMD_ENTER_DO_NOTHING.md` |
| BB-BUG-022 | Editor rows span the pane (`w_full`); a click past the text lands on the line's end | FIXED | red-first window test + `clickpast` readback | `D2/S06/BB_BUG_022_A_CLICK_RIGHT_OF_A_LINES_END_DOES_NOTHING.md` |
| BB-BUG-023 | One `EXPLORER  TESTS` strip; the chosen tab in the `chosen` pink | FIXED | window + unit test | `D2/S06/BB_BUG_023_THE_SIDEBAR_HEADER_DOES_NOT_SHOW_WHICH_SIDE_IS_CHOSEN.md` |
| BB-BUG-024 | Namespace, class and method heads fold per project (`Group`, `TestsState::toggle`) | FIXED | window test | `D2/S06/BB_BUG_024_TEST_GROUPS_DO_NOT_COLLAPSE.md` |
| BB-BUG-025 | A row click opens the method (`bb_testkit::find_source`, off the UI thread) | FIXED | 7 unit + window test + readback | `D2/S07/BB_BUG_025_A_CLICK_ON_A_TEST_DOES_NOT_OPEN_IT.md` |
| BB-BUG-026 | ▶ turns ■, Run turns ■ Stop while running; toolbar stop only during a run | FIXED | window test + process readback | `D2/S07/BB_BUG_026_A_RUNNING_TEST_CANNOT_BE_STOPPED.md` |
| BB-BUG-027 | `OwnedTempDir` (drop, quit, sweep by pid); `bb sweep` after each gate | FIXED | unit tests + 3156 swept + leak probe | `D2/S07/BB_BUG_027_BACKBACK_LEAVES_TEMP_FOLDERS_BEHIND.md` |
| BB-BUG-028 | Cmd+C/X/V: selection, else the whole line; a line pastes above | FIXED | red-first window test + 3 unit tests | `D2/S07/BB_BUG_028_CMD_C_DOES_NOT_COPY.md` |
| BB-BUG-029 | `Explorer::reveal`; the side panel follows each newly active file | FIXED | 2 unit + red-first window test + readback | `D3/S08/BB_BUG_029_THE_EXPLORER_DOES_NOT_FOLLOW_THE_ACTIVE_FILE.md` |
| BB-BUG-030 | `binary or non-UTF-8 file: not shown as text` | FIXED | readback | `D3/S08/BB_BUG_030_A_BINARY_FILE_OPENS_WITH_A_RAW_DECODER_ERROR.md` |
