# Bugfix index

| Id | Fix | Status | Proof | Path |
|---|---|---|---|---|
| BB-BUG-003 | Panel removed in PR #9; running a fresh build shows no panel | FIXED | guard tests + no-MCP screenshot in #9 | `S01/BB_BUG_003_MCP_PANEL_STILL_SHOWN.md` |
| BB-BUG-005 | Caret is a quad over one shaped line; clicks and popups measure the same line | FIXED | 4 red->green window tests + pixel-diff readback | `S02/BB_BUG_005_TEXT_SHIFTS_WHEN_THE_CURSOR_MOVES.md` |
| BB-BUG-001 | Window menu + Cmd+M/Cmd+H/Cmd+Alt+H | FIXED | 2 red-first tests + readback (window minimized) | `S01/BB_BUG_001_CMD_M_DOES_NOT_MINIMIZE.md` |
| BB-BUG-007 | Home/End smart, Cmd+arrows, Shift extends | FIXED | 5 red-first tests + readback cursor log | `S02/BB_BUG_007_HOME_AND_END_DO_NOTHING.md` |
| BB-BUG-006 | Finder launch opens the recent workspace; Cmd+O; recent file | FIXED | red-first resolve test + readback | `S02/BB_BUG_006_APP_OPENS_THE_DISK_ROOT.md` |
| BB-BUG-002 | One Typography, set only at the window root; Settings, zoom | FIXED | source scan + window tests + live capture | `S01/BB_BUG_002_NOT_ONE_FONT_EVERYWHERE.md` |
| BB-BUG-008 | `locate` also checks ~/.local/share/backback/netcoredbg and Homebrew | FIXED | red-first locate test + installed-app readback | `S02/BB_BUG_008_F5_FROM_THE_DOCK_FINDS_NO_NETCOREDBG.md` |
| BB-BUG-009 | Adopt the login shell PATH at start (VS Code style); .NET folders as fallback | FIXED | 4 tests + launchd-env readback | `S03/BB_BUG_009_AN_APP_FROM_THE_DOCK_FINDS_NO_DOTNET.md` |
| BB-BUG-010 | `\r\n` is one break: arrows step over it, Backspace/Delete remove it whole | FIXED | 3 red-first tests + byte readback | `S03/BB_BUG_010_RIGHT_AT_A_LINE_END_SPLITS_A_CRLF.md` |
| BB-BUG-011 | Option+Left/Right/Backspace/Delete work by word (VS Code / IntelliJ stops) | FIXED | 6 red-first tests + window test | `S03/BB_BUG_011_OPTION_ARROWS_MOVE_ONE_CHAR_NOT_ONE_WORD.md` |
| BB-BUG-012 | Ctrl+A/E/B/F/P/N/H/D work as in Cocoa and VS Code on macOS | FIXED | red-first window test | `S03/BB_BUG_012_THE_MACOS_CTRL_TEXT_KEYS_DO_NOTHING.md` |
| BB-BUG-013 | A per-tab `scroll_x`: clipped text viewport per row, caret and clicks offset, wheel scrolls, the caret kept in sight on moves | FIXED | red-first window test + readback on the headmate's file | `S04/BB_BUG_013_LONG_LINES_ARE_CLIPPED_WITH_NO_HORIZONTAL_SCROLL.md` |
| BB-BUG-014 | `delete_to_line_start` / `delete_to_line_end` on Cmd+Backspace / Cmd+Delete | FIXED | red-first unit + window tests | `S04/BB_BUG_014_CMD_BACKSPACE_DOES_NOTHING.md` |
| BB-BUG-015 | Tab indents and Shift+Tab outdents the selected lines, the selection kept, one undo step | FIXED | red-first unit + window tests | `S04/BB_BUG_015_TAB_WITH_A_SELECTION_REPLACES_IT.md` |
