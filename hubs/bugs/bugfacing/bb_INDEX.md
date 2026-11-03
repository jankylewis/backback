# Bugfacing index

Bugs a user met in the app. Entries: 15 — FIXED 12, OPEN 3.

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
| BB-BUG-013 | Long lines are clipped at the pane's edge; no horizontal scroll | FIXED | rows had no horizontal offset; the list clipped them | yes | `S04/BB_BUG_013_LONG_LINES_ARE_CLIPPED_WITH_NO_HORIZONTAL_SCROLL.md` |
| BB-BUG-014 | Cmd+Backspace does nothing | OPEN | the key map has no Cmd+Backspace arm | no | `S04/BB_BUG_014_CMD_BACKSPACE_DOES_NOTHING.md` |
| BB-BUG-015 | Tab with a selection replaces it; Shift+Tab inserts a tab | OPEN | Tab inserts a char over the selection; no indent command | no | `S04/BB_BUG_015_TAB_WITH_A_SELECTION_REPLACES_IT.md` |
