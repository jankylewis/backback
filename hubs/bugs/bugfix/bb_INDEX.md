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
| BB-BUG-021 | `move_lines` / `copy_lines` on Alt / Shift+Alt arrows; one undo step; breaks kept | FIXED | 5 unit + window test | `D2/S06/BB_BUG_021_MOVE_COPY_AND_SELECT_LINE_KEYS_DO_NOTHING.md` |
| BB-BUG-032 | Lines drawn and measured up to 10 000 chars (`RENDER_CAP`) | FIXED | unit test + readback 42 s -> 2 s | `D3/S08/BB_BUG_032_A_VERY_LONG_LINE_FREEZES_THE_EDITOR.md` |
| BB-BUG-033 | Left/Right/Backspace/Delete step one grapheme (`graphemes.rs`) | FIXED | unit test + readback col 22 -> 21 | `D3/S09/BB_BUG_033_THE_CURSOR_STOPS_INSIDE_A_COMBINED_CHARACTER.md` |
| BB-BUG-034 | `DiskGuard`: a save over a newer file is refused once, then overwrites | FIXED | unit test + readback | `D3/S09/BB_BUG_034_A_SAVE_OVERWRITES_ANOTHER_TOOLS_CHANGE.md` |
| BB-BUG-035 | Clean tabs take the disk's text (2 s check, one undo step) | FIXED | unit test + readback | `D3/S09/BB_BUG_035_A_CLEAN_TAB_KEEPS_STALE_TEXT_AFTER_AN_OUTSIDE_CHANGE.md` |
| BB-BUG-036 | Cmd+P bound to Go to File; Cmd+Shift+O awaits a ruling | FIXED | unit test + readback | `D3/S09/BB_BUG_036_CMD_P_DOES_NOT_OPEN_GO_TO_FILE.md` |
| BB-BUG-037 | The 2 s disk check re-reads loaded folders, merged by path | FIXED | unit test + readback | `D3/S10/BB_BUG_037_THE_EXPLORER_MISSES_FILES_MADE_OR_REMOVED_OUTSIDE.md` |
| BB-BUG-038 | One `backback-intel-<pid>-<n>` folder per worker | FIXED | red-first unit test + gate | `D3/S10/BB_BUG_038_A_SECOND_WINDOW_WIPES_THE_FIRSTS_ROSLYN_FOLDER.md` |
| BB-BUG-039 | Pinned tabs, a tab menu and the bulk closes, VS Code keys | FIXED | unit + window tests + readback | `D3/S10/BB_BUG_039_NO_PINNED_TABS_AND_NO_BULK_CLOSE.md` |
| BB-BUG-040 | Two editor groups; drag, menu rows and keys move a tab | FIXED | unit + window tests + readback | `D3/S10/BB_BUG_040_NO_SPLIT_EDITOR.md` |
| BB-BUG-041 | The save prompt holds the file's path | FIXED | red-first unit test + readback | `D3/S11/BB_BUG_041_THE_SAVE_PROMPT_ACTS_ON_THE_WRONG_TAB_AFTER_A_MOVE.md` |
| BB-BUG-042 | A press outside closes the box; a row press opens it | FIXED | window test (mutation-checked) + readback | `D3/S11/BB_BUG_042_GO_TO_FILE_AND_FIND_IN_FILES_CANNOT_BE_CLOSED_BY_A_CLICK.md` |
| BB-BUG-043 | Status bar names the build; fix installed; split-layout window test | FIXED | window test + build stamp | `D3/S11/BB_BUG_043_GO_TO_FILE_STILL_DID_NOT_CLOSE_ON_A_CLICK_OUTSIDE.md` |
| BB-BUG-044 | Drag lines (`Lines`); results scroll both ways, match first | FIXED | window tests (mutation-checked) | `D3/S11/BB_BUG_044_PANES_AND_THE_RESULTS_PANEL_COULD_NOT_BE_RESIZED_OR_SCROLLED.md` |
| BB-BUG-045 | Up to three groups; each line drags; groups close up | FIXED | model + window tests | `D4/S12/BB_BUG_045_ONLY_TWO_EDITOR_PANES.md` |
| BB-BUG-046 | Theme text colour; terminal top edge drags | FIXED | contrast test (mutation-checked) + window test | `D4/S12/BB_BUG_046_TERMINAL_TEXT_UNREADABLE_AND_HEIGHT_FIXED.md` |
| BB-BUG-047 | ✕ on every bottom panel; View → Terminal | FIXED | window tests + menu unit test | `D4/S12/BB_BUG_047_PANELS_COULD_NOT_BE_CLOSED_AND_NO_TERMINAL_MENU.md` |
| BB-BUG-048 | Window titled by folder; File > New Window | FIXED | window test (mutation-checked) + menu/key test | `D4/S12/BB_BUG_048_NO_NEW_WINDOW_AND_WINDOWS_NOT_IN_THE_WINDOW_MENU.md` |
| BB-BUG-049 | Scroll clamped every frame | FIXED | window test (mutation-checked) | `D4/S13/BB_BUG_049_A_WIDER_WINDOW_KEPT_THE_TEXT_SCROLLED_AND_CUT.md` |
| BB-BUG-050 | Explorer right-click menu; Trash with a seam | FIXED | unit + window tests | `D4/S13/BB_BUG_050_NO_NEW_RENAME_OR_DELETE_IN_THE_EXPLORER.md` |
| BB-BUG-051 | Vertical + horizontal scroll bars, draggable | FIXED | unit + window tests | `D4/S13/BB_BUG_051_NO_SCROLL_BARS_IN_THE_EDITOR.md` |
| BB-BUG-052 | Open Terminal Here, Copy/Paste, Copy Path | FIXED | unit + real-shell + window tests (mutation-checked) | `D4/S13/BB_BUG_052_NO_COPY_PASTE_OR_TERMINAL_HERE_IN_THE_EXPLORER.md` |
| BB-BUG-053 | Mouse drag selects (alacritty selection); Cmd+C copies | FIXED | session + window tests (mutation-checked) | `D4/S14/BB_BUG_053_TERMINAL_TEXT_COULD_NOT_BE_SELECTED.md` |
| BB-BUG-054 | `selection` token, plain text on it | FIXED | contrast + run tests (mutation-checked) | `D4/S14/BB_BUG_054_SELECTED_TEXT_WAS_UNREADABLE.md` |
| BB-BUG-055 | `after_key` scrolls to the cursor line (Nearest) | FIXED | window test, red first | `D4/S14/BB_BUG_055_THE_VIEW_DID_NOT_FOLLOW_THE_CURSOR.md` |
| BB-BUG-056 | one `editor_area` width for scroll and terminal | FIXED | window tests both sides, red first | `D4/S14/BB_BUG_056_THE_LINE_END_HID_UNDER_A_RIGHT_DOCKED_PANEL.md` |
| BB-BUG-057 | one caret model for every quick box: paste, arrows, Delete | FIXED | window test, red first | `D4/S15/BB_BUG_057_THE_FIND_BOX_TOOK_NO_PASTE_AND_HAD_NO_CARET.md` |
