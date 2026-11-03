# Bugfacing index

Bugs a user met in the app. Entries: 23 — FIXED 21, OPEN 2.

| Id | Bug | Status | Cause | Fix | Path |
|---|---|---|---|---|---|
| BB-BUG-001 | Cmd+M does not minimize the window | FIXED | missing binding | yes | `S01/BB_BUG_001_CMD_M_DOES_NOT_MINIMIZE.md` |
| BB-BUG-002 | Explorer and side panels not in Menlo; sizes mixed | FIXED | literal fonts per view | yes | `S01/BB_BUG_002_NOT_ONE_FONT_EVERYWHERE.md` |
| BB-BUG-003 | MCP HARNESS panel still shown after its removal | FIXED | BB-BLD-001 | yes | `S01/BB_BUG_003_MCP_PANEL_STILL_SHOWN.md` |
| BB-BUG-004 | Test Explorer did not respond to clicks | OPEN | not proven | no | `S01/BB_BUG_004_TEST_EXPLORER_CLICKS_NO_RESPONSE.md` |
| BB-BUG-005 | Text shifts one column when the cursor moves | FIXED | BB-UIX-001 | yes | `S02/BB_BUG_005_TEXT_SHIFTS_WHEN_THE_CURSOR_MOVES.md` |
| BB-BUG-006 | Opened from the Dock or Finder, the app shows the disk root | FIXED | no Finder-launch rule | yes | `S02/BB_BUG_006_APP_OPENS_THE_DISK_ROOT.md` |
| BB-BUG-007 | Home and End (and Cmd+Left/Right) do nothing in the editor | FIXED | missing keys | yes | `S02/BB_BUG_007_HOME_AND_END_DO_NOTHING.md` |
| BB-BUG-008 | F5 in the installed app finds no netcoredbg | FIXED | Dock launch has no PATH or override | yes | `S02/BB_BUG_008_F5_FROM_THE_DOCK_FINDS_NO_NETCOREDBG.md` |
| BB-BUG-009 | An app opened from the Dock finds no `dotnet` | FIXED | launchd PATH | yes | `S03/BB_BUG_009_AN_APP_FROM_THE_DOCK_FINDS_NO_DOTNET.md` |
| BB-BUG-010 | Right at a line end splits a CRLF line break | FIXED | char-by-char movement over `\r\n` | yes | `S03/BB_BUG_010_RIGHT_AT_A_LINE_END_SPLITS_A_CRLF.md` |
| BB-BUG-011 | Option+arrows move one char, not one word | FIXED | Option was ignored by the editor key map | yes | `S03/BB_BUG_011_OPTION_ARROWS_MOVE_ONE_CHAR_NOT_ONE_WORD.md` |
| BB-BUG-012 | The macOS Ctrl text keys do nothing | FIXED | the key map had no Ctrl arms | yes | `S03/BB_BUG_012_THE_MACOS_CTRL_TEXT_KEYS_DO_NOTHING.md` |
| BB-BUG-013 | Long lines are clipped at the pane's edge; no horizontal scroll | FIXED | rows had no horizontal offset; the list clipped them | yes | `D2/S04/BB_BUG_013_LONG_LINES_ARE_CLIPPED_WITH_NO_HORIZONTAL_SCROLL.md` |
| BB-BUG-014 | Cmd+Backspace does nothing | FIXED | the key map had no Cmd+Backspace arm (BB-UIX-007) | yes | `D2/S04/BB_BUG_014_CMD_BACKSPACE_DOES_NOTHING.md` |
| BB-BUG-015 | Tab with a selection replaces it; Shift+Tab inserts a tab | FIXED | Tab typed its char over the selection; no indent command (BB-UIX-007) | yes | `D2/S04/BB_BUG_015_TAB_WITH_A_SELECTION_REPLACES_IT.md` |
| BB-BUG-016 | Cmd+A does not select all | FIXED | no Cmd+A arm in the key map (BB-UIX-008) | yes | `D2/S04/BB_BUG_016_CMD_A_DOES_NOT_SELECT_ALL.md` |
| BB-BUG-017 | Cmd+/ does not toggle a line comment | FIXED | no comment command, no Cmd+/ arm (BB-UIX-008) | yes | `D2/S05/BB_BUG_017_CMD_SLASH_DOES_NOT_TOGGLE_A_LINE_COMMENT.md` |
| BB-BUG-018 | The rename box appends to the old name | FIXED | the prefilled name was not selected (BB-UIX-009) | yes | `D2/S05/BB_BUG_018_THE_RENAME_BOX_APPENDS_TO_THE_OLD_NAME.md` |
| BB-BUG-019 | Problems keep stale diagnostics after a rename | FIXED | an edit pulled again only its own file (BB-LSP-007) | yes | `D2/S05/BB_BUG_019_PROBLEMS_KEEP_STALE_DIAGNOSTICS_AFTER_A_RENAME.md` |
| BB-BUG-020 | Cmd+Shift+K and Cmd+Enter do nothing | FIXED | the key map lacked the line commands (BB-UIX-010) | yes | `D2/S05/BB_BUG_020_CMD_SHIFT_K_AND_CMD_ENTER_DO_NOTHING.md` |
| BB-BUG-021 | The move, copy and select-line keys do nothing | OPEN | the key map lacked the line commands (BB-UIX-010) | no | `D2/S06/BB_BUG_021_MOVE_COPY_AND_SELECT_LINE_KEYS_DO_NOTHING.md` |
| BB-BUG-022 | A click right of a line's end does nothing | FIXED | editor rows were only as wide as their text (BB-UIX-011) | yes | `D2/S06/BB_BUG_022_A_CLICK_RIGHT_OF_A_LINES_END_DOES_NOTHING.md` |
| BB-BUG-023 | The sidebar header does not show which side is chosen | FIXED | design: a title and a link | yes | `D2/S06/BB_BUG_023_THE_SIDEBAR_HEADER_DOES_NOT_SHOW_WHICH_SIDE_IS_CHOSEN.md` |
