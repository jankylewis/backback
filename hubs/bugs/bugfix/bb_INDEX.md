# Bugfix index

| Id | Fix | Status | Proof | Path |
|---|---|---|---|---|
| BB-BUG-003 | Panel removed in PR #9; running a fresh build shows no panel | FIXED | guard tests + no-MCP screenshot in #9 | `S01/BB_BUG_003_MCP_PANEL_STILL_SHOWN.md` |
| BB-BUG-005 | Caret is a quad over one shaped line; clicks and popups measure the same line | FIXED | 4 red->green window tests + pixel-diff readback | `S02/BB_BUG_005_TEXT_SHIFTS_WHEN_THE_CURSOR_MOVES.md` |
| BB-BUG-001 | Window menu + Cmd+M/Cmd+H/Cmd+Alt+H | FIXED | 2 red-first tests + readback (window minimized) | `S01/BB_BUG_001_CMD_M_DOES_NOT_MINIMIZE.md` |
| BB-BUG-007 | Home/End smart, Cmd+arrows, Shift extends | FIXED | 5 red-first tests + readback cursor log | `S02/BB_BUG_007_HOME_AND_END_DO_NOTHING.md` |
| BB-BUG-006 | Finder launch opens the recent workspace; Cmd+O; recent file | FIXED | red-first resolve test + readback | `S02/BB_BUG_006_APP_OPENS_THE_DISK_ROOT.md` |
