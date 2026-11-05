# Bugfacing index

Bugs a user met in the app. Entries: 52 — FIXED 50, OBSERVED 2.

| Id | Bug | Status | Cause | Fix | Path |
|---|---|---|---|---|---|
| BB-BUG-001 | Cmd+M does not minimize the window | FIXED | missing binding | yes | `D1/S01/BB_BUG_001_CMD_M_DOES_NOT_MINIMIZE.md` |
| BB-BUG-002 | Explorer and side panels not in Menlo; sizes mixed | FIXED | literal fonts per view | yes | `D1/S01/BB_BUG_002_NOT_ONE_FONT_EVERYWHERE.md` |
| BB-BUG-003 | MCP HARNESS panel still shown after its removal | FIXED | BB-BLD-001 | yes | `D1/S01/BB_BUG_003_MCP_PANEL_STILL_SHOWN.md` |
| BB-BUG-004 | Test Explorer did not respond to clicks | OBSERVED | not reproduced: hand clicks run tests (BB-BUG-026's report); the first report's input never arrived (BB-TG-007) | no | `D1/S01/BB_BUG_004_TEST_EXPLORER_CLICKS_NO_RESPONSE.md` |
| BB-BUG-005 | Text shifts one column when the cursor moves | FIXED | BB-UIX-001 | yes | `D1/S02/BB_BUG_005_TEXT_SHIFTS_WHEN_THE_CURSOR_MOVES.md` |
| BB-BUG-006 | Opened from the Dock or Finder, the app shows the disk root | FIXED | no Finder-launch rule | yes | `D1/S02/BB_BUG_006_APP_OPENS_THE_DISK_ROOT.md` |
| BB-BUG-007 | Home and End (and Cmd+Left/Right) do nothing in the editor | FIXED | missing keys | yes | `D1/S02/BB_BUG_007_HOME_AND_END_DO_NOTHING.md` |
| BB-BUG-008 | F5 in the installed app finds no netcoredbg | FIXED | Dock launch has no PATH or override | yes | `D1/S02/BB_BUG_008_F5_FROM_THE_DOCK_FINDS_NO_NETCOREDBG.md` |
| BB-BUG-009 | An app opened from the Dock finds no `dotnet` | FIXED | launchd PATH | yes | `D1/S03/BB_BUG_009_AN_APP_FROM_THE_DOCK_FINDS_NO_DOTNET.md` |
| BB-BUG-010 | Right at a line end splits a CRLF line break | FIXED | char-by-char movement over `\r\n` | yes | `D1/S03/BB_BUG_010_RIGHT_AT_A_LINE_END_SPLITS_A_CRLF.md` |
| BB-BUG-011 | Option+arrows move one char, not one word | FIXED | Option was ignored by the editor key map | yes | `D1/S03/BB_BUG_011_OPTION_ARROWS_MOVE_ONE_CHAR_NOT_ONE_WORD.md` |
| BB-BUG-012 | The macOS Ctrl text keys do nothing | FIXED | the key map had no Ctrl arms | yes | `D1/S03/BB_BUG_012_THE_MACOS_CTRL_TEXT_KEYS_DO_NOTHING.md` |
| BB-BUG-013 | Long lines are clipped at the pane's edge; no horizontal scroll | FIXED | rows had no horizontal offset; the list clipped them | yes | `D2/S04/BB_BUG_013_LONG_LINES_ARE_CLIPPED_WITH_NO_HORIZONTAL_SCROLL.md` |
| BB-BUG-014 | Cmd+Backspace does nothing | FIXED | the key map had no Cmd+Backspace arm (BB-UIX-007) | yes | `D2/S04/BB_BUG_014_CMD_BACKSPACE_DOES_NOTHING.md` |
| BB-BUG-015 | Tab with a selection replaces it; Shift+Tab inserts a tab | FIXED | Tab typed its char over the selection; no indent command (BB-UIX-007) | yes | `D2/S04/BB_BUG_015_TAB_WITH_A_SELECTION_REPLACES_IT.md` |
| BB-BUG-016 | Cmd+A does not select all | FIXED | no Cmd+A arm in the key map (BB-UIX-008) | yes | `D2/S04/BB_BUG_016_CMD_A_DOES_NOT_SELECT_ALL.md` |
| BB-BUG-017 | Cmd+/ does not toggle a line comment | FIXED | no comment command, no Cmd+/ arm (BB-UIX-008) | yes | `D2/S05/BB_BUG_017_CMD_SLASH_DOES_NOT_TOGGLE_A_LINE_COMMENT.md` |
| BB-BUG-018 | The rename box appends to the old name | FIXED | the prefilled name was not selected (BB-UIX-009) | yes | `D2/S05/BB_BUG_018_THE_RENAME_BOX_APPENDS_TO_THE_OLD_NAME.md` |
| BB-BUG-019 | Problems keep stale diagnostics after a rename | FIXED | an edit pulled again only its own file (BB-LSP-007) | yes | `D2/S05/BB_BUG_019_PROBLEMS_KEEP_STALE_DIAGNOSTICS_AFTER_A_RENAME.md` |
| BB-BUG-020 | Cmd+Shift+K and Cmd+Enter do nothing | FIXED | the key map lacked the line commands (BB-UIX-010) | yes | `D2/S05/BB_BUG_020_CMD_SHIFT_K_AND_CMD_ENTER_DO_NOTHING.md` |
| BB-BUG-021 | The move, copy and select-line keys do nothing | FIXED | the key map lacked the line commands (BB-UIX-010) | yes | `D2/S06/BB_BUG_021_MOVE_COPY_AND_SELECT_LINE_KEYS_DO_NOTHING.md` |
| BB-BUG-022 | A click right of a line's end does nothing | FIXED | editor rows were only as wide as their text (BB-UIX-011) | yes | `D2/S06/BB_BUG_022_A_CLICK_RIGHT_OF_A_LINES_END_DOES_NOTHING.md` |
| BB-BUG-023 | The sidebar header does not show which side is chosen | FIXED | design: a title and a link | yes | `D2/S06/BB_BUG_023_THE_SIDEBAR_HEADER_DOES_NOT_SHOW_WHICH_SIDE_IS_CHOSEN.md` |
| BB-BUG-024 | Test groups do not collapse | FIXED | a fold marker with no fold (BB-UIX-012) | yes | `D2/S06/BB_BUG_024_TEST_GROUPS_DO_NOT_COLLAPSE.md` |
| BB-BUG-025 | A click on a test does not open it | FIXED | design: not built | yes | `D2/S07/BB_BUG_025_A_CLICK_ON_A_TEST_DOES_NOT_OPEN_IT.md` |
| BB-BUG-026 | A running test cannot be stopped | FIXED | the stop was far from the run (BB-UIX-013) | yes | `D2/S07/BB_BUG_026_A_RUNNING_TEST_CANNOT_BE_STOPPED.md` |
| BB-BUG-027 | backback leaves temp folders behind | FIXED | temp folders outlive their process (BB-PRC-004) | yes | `D2/S07/BB_BUG_027_BACKBACK_LEAVES_TEMP_FOLDERS_BEHIND.md` |
| BB-BUG-028 | Cmd+C does not copy | FIXED | design: not built | yes | `D2/S07/BB_BUG_028_CMD_C_DOES_NOT_COPY.md` |
| BB-BUG-029 | The Explorer does not follow the active file | FIXED | design: not built | yes | `D3/S08/BB_BUG_029_THE_EXPLORER_DOES_NOT_FOLLOW_THE_ACTIVE_FILE.md` |
| BB-BUG-030 | A binary file opens with a raw decoder error | FIXED | the decoder's text | yes | `D3/S08/BB_BUG_030_A_BINARY_FILE_OPENS_WITH_A_RAW_DECODER_ERROR.md` |
| BB-BUG-031 | The Explorer seems to hide files | OBSERVED | not a bug: the folders hold only caches (BB-UIX-014) | no | `D3/S08/BB_BUG_031_THE_EXPLORER_SEEMS_TO_HIDE_FILES.md` |
| BB-BUG-032 | A very long line freezes the editor | FIXED | every frame shaped and styled the whole line (BB-PRF-004) | yes | `D3/S08/BB_BUG_032_A_VERY_LONG_LINE_FREEZES_THE_EDITOR.md` |
| BB-BUG-033 | The cursor stops inside a combined character | FIXED | moves stepped one code point, not one grapheme | yes | `D3/S09/BB_BUG_033_THE_CURSOR_STOPS_INSIDE_A_COMBINED_CHARACTER.md` |
| BB-BUG-034 | A save overwrites another tool's change | FIXED | save never compared the file with what it had read (BB-UIX-015) | yes | `D3/S09/BB_BUG_034_A_SAVE_OVERWRITES_ANOTHER_TOOLS_CHANGE.md` |
| BB-BUG-035 | A clean tab keeps stale text after an outside change | FIXED | nothing compared the file with what was read (BB-UIX-015) | yes | `D3/S09/BB_BUG_035_A_CLEAN_TAB_KEEPS_STALE_TEXT_AFTER_AN_OUTSIDE_CHANGE.md` |
| BB-BUG-036 | Cmd+P does not open Go to File | FIXED | the key map lacked VS Code's keys (BB-UIX-010) | yes | `D3/S09/BB_BUG_036_CMD_P_DOES_NOT_OPEN_GO_TO_FILE.md` |
| BB-BUG-037 | The Explorer misses files made or removed outside | FIXED | each folder was read once (BB-UIX-016) | yes | `D3/S10/BB_BUG_037_THE_EXPLORER_MISSES_FILES_MADE_OR_REMOVED_OUTSIDE.md` |
| BB-BUG-038 | A second window wipes the first's Roslyn folder | FIXED | two workers shared one owned temp folder (BB-PRC-005) | yes | `D3/S10/BB_BUG_038_A_SECOND_WINDOW_WIPES_THE_FIRSTS_ROSLYN_FOLDER.md` |
| BB-BUG-039 | No pinned tabs and no bulk close | FIXED | not built (VS Code tab menu) | yes | `D3/S10/BB_BUG_039_NO_PINNED_TABS_AND_NO_BULK_CLOSE.md` |
| BB-BUG-040 | No split editor: a tab cannot be dragged to the right side | FIXED | not built (VS Code split editor) | yes | `D3/S10/BB_BUG_040_NO_SPLIT_EDITOR.md` |
| BB-BUG-041 | The save prompt acts on the wrong tab after tabs move | FIXED | the prompt held an index, not a file | yes | `D3/S11/BB_BUG_041_THE_SAVE_PROMPT_ACTS_ON_THE_WRONG_TAB_AFTER_A_MOVE.md` |
| BB-BUG-042 | Go to File / Find in Files cannot be closed by a click outside | FIXED | only Escape closed it, and only with key focus | yes | `D3/S11/BB_BUG_042_GO_TO_FILE_AND_FIND_IN_FILES_CANNOT_BE_CLOSED_BY_A_CLICK.md` |
| BB-BUG-043 | Go to File still did not close on a click outside | FIXED | the fix was merged, not installed (BB-BLD-003) | yes | `D3/S11/BB_BUG_043_GO_TO_FILE_STILL_DID_NOT_CLOSE_ON_A_CLICK_OUTSIDE.md` |
| BB-BUG-044 | The panes and the results panel could not be resized; long results were cut off | FIXED | not built (fixed sizes, clipped rows) | yes | `D3/S11/BB_BUG_044_PANES_AND_THE_RESULTS_PANEL_COULD_NOT_BE_RESIZED_OR_SCROLLED.md` |
| BB-BUG-045 | Only two editor panes; three files could not show side by side | FIXED | not built (two groups only) | yes | `D4/S12/BB_BUG_045_ONLY_TWO_EDITOR_PANES.md` |
| BB-BUG-046 | The terminal's text was unreadable and its height fixed | FIXED | no text colour set (gpui default black); fixed height | yes | `D4/S12/BB_BUG_046_TERMINAL_TEXT_UNREADABLE_AND_HEIGHT_FIXED.md` |
| BB-BUG-047 | Panels had no close button; the View menu had no Terminal | FIXED | not built (key-only close; no menu item) | yes | `D4/S12/BB_BUG_047_PANELS_COULD_NOT_BE_CLOSED_AND_NO_TERMINAL_MENU.md` |
| BB-BUG-048 | No New Window; open windows were not in the Window menu | FIXED | not built (untitled windows; no New Window) | yes | `D4/S12/BB_BUG_048_NO_NEW_WINDOW_AND_WINDOWS_NOT_IN_THE_WINDOW_MENU.md` |
| BB-BUG-049 | A wider window kept the text scrolled sideways and cut off | FIXED | scroll clamped only on a swipe | yes | `D4/S13/BB_BUG_049_A_WIDER_WINDOW_KEPT_THE_TEXT_SCROLLED_AND_CUT.md` |
| BB-BUG-050 | No New File, New Folder, Rename or Delete in the Explorer | FIXED | not built | yes | `D4/S13/BB_BUG_050_NO_NEW_RENAME_OR_DELETE_IN_THE_EXPLORER.md` |
| BB-BUG-051 | The editor had no scroll bars | FIXED | not built | yes | `D4/S13/BB_BUG_051_NO_SCROLL_BARS_IN_THE_EDITOR.md` |
| BB-BUG-052 | No Copy, Paste or Open Terminal Here in the Explorer | FIXED | not built | yes | `D4/S13/BB_BUG_052_NO_COPY_PASTE_OR_TERMINAL_HERE_IN_THE_EXPLORER.md` |
