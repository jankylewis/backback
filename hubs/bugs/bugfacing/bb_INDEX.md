# Bugfacing index

Bugs a user met in the app. Entries: 92 — FIXED 88, OBSERVED 4.

| Id | Bug | Status | Cause | Fix | Path |
|---|---|---|---|---|---|
| BB-BUG-001 | Cmd+M does not minimize the window | FIXED | missing binding | yes | `C1/D1/S01/BB_BUG_001_CMD_M_DOES_NOT_MINIMIZE.md` |
| BB-BUG-002 | Explorer and side panels not in Menlo; sizes mixed | FIXED | literal fonts per view | yes | `C1/D1/S01/BB_BUG_002_NOT_ONE_FONT_EVERYWHERE.md` |
| BB-BUG-003 | MCP HARNESS panel still shown after its removal | FIXED | BB-BLD-001 | yes | `C1/D1/S01/BB_BUG_003_MCP_PANEL_STILL_SHOWN.md` |
| BB-BUG-004 | Test Explorer did not respond to clicks | OBSERVED | not reproduced: hand clicks run tests (BB-BUG-026's report); ▶ window test on master 2026-11-20; the first report's input never arrived (BB-TG-007) | no | `C1/D1/S01/BB_BUG_004_TEST_EXPLORER_CLICKS_NO_RESPONSE.md` |
| BB-BUG-005 | Text shifts one column when the cursor moves | FIXED | BB-UIX-001 | yes | `C1/D1/S02/BB_BUG_005_TEXT_SHIFTS_WHEN_THE_CURSOR_MOVES.md` |
| BB-BUG-006 | Opened from the Dock or Finder, the app shows the disk root | FIXED | no Finder-launch rule | yes | `C1/D1/S02/BB_BUG_006_APP_OPENS_THE_DISK_ROOT.md` |
| BB-BUG-007 | Home and End (and Cmd+Left/Right) do nothing in the editor | FIXED | missing keys | yes | `C1/D1/S02/BB_BUG_007_HOME_AND_END_DO_NOTHING.md` |
| BB-BUG-008 | F5 in the installed app finds no netcoredbg | FIXED | Dock launch has no PATH or override | yes | `C1/D1/S02/BB_BUG_008_F5_FROM_THE_DOCK_FINDS_NO_NETCOREDBG.md` |
| BB-BUG-009 | An app opened from the Dock finds no `dotnet` | FIXED | launchd PATH | yes | `C1/D1/S03/BB_BUG_009_AN_APP_FROM_THE_DOCK_FINDS_NO_DOTNET.md` |
| BB-BUG-010 | Right at a line end splits a CRLF line break | FIXED | char-by-char movement over `\r\n` | yes | `C1/D1/S03/BB_BUG_010_RIGHT_AT_A_LINE_END_SPLITS_A_CRLF.md` |
| BB-BUG-011 | Option+arrows move one char, not one word | FIXED | Option was ignored by the editor key map | yes | `C1/D1/S03/BB_BUG_011_OPTION_ARROWS_MOVE_ONE_CHAR_NOT_ONE_WORD.md` |
| BB-BUG-012 | The macOS Ctrl text keys do nothing | FIXED | the key map had no Ctrl arms | yes | `C1/D1/S03/BB_BUG_012_THE_MACOS_CTRL_TEXT_KEYS_DO_NOTHING.md` |
| BB-BUG-013 | Long lines are clipped at the pane's edge; no horizontal scroll | FIXED | rows had no horizontal offset; the list clipped them | yes | `C1/D2/S04/BB_BUG_013_LONG_LINES_ARE_CLIPPED_WITH_NO_HORIZONTAL_SCROLL.md` |
| BB-BUG-014 | Cmd+Backspace does nothing | FIXED | the key map had no Cmd+Backspace arm (BB-UIX-007) | yes | `C1/D2/S04/BB_BUG_014_CMD_BACKSPACE_DOES_NOTHING.md` |
| BB-BUG-015 | Tab with a selection replaces it; Shift+Tab inserts a tab | FIXED | Tab typed its char over the selection; no indent command (BB-UIX-007) | yes | `C1/D2/S04/BB_BUG_015_TAB_WITH_A_SELECTION_REPLACES_IT.md` |
| BB-BUG-016 | Cmd+A does not select all | FIXED | no Cmd+A arm in the key map (BB-UIX-008) | yes | `C1/D2/S04/BB_BUG_016_CMD_A_DOES_NOT_SELECT_ALL.md` |
| BB-BUG-017 | Cmd+/ does not toggle a line comment | FIXED | no comment command, no Cmd+/ arm (BB-UIX-008) | yes | `C1/D2/S05/BB_BUG_017_CMD_SLASH_DOES_NOT_TOGGLE_A_LINE_COMMENT.md` |
| BB-BUG-018 | The rename box appends to the old name | FIXED | the prefilled name was not selected (BB-UIX-009) | yes | `C1/D2/S05/BB_BUG_018_THE_RENAME_BOX_APPENDS_TO_THE_OLD_NAME.md` |
| BB-BUG-019 | Problems keep stale diagnostics after a rename | FIXED | an edit pulled again only its own file (BB-LSP-007) | yes | `C1/D2/S05/BB_BUG_019_PROBLEMS_KEEP_STALE_DIAGNOSTICS_AFTER_A_RENAME.md` |
| BB-BUG-020 | Cmd+Shift+K and Cmd+Enter do nothing | FIXED | the key map lacked the line commands (BB-UIX-010) | yes | `C1/D2/S05/BB_BUG_020_CMD_SHIFT_K_AND_CMD_ENTER_DO_NOTHING.md` |
| BB-BUG-021 | The move, copy and select-line keys do nothing | FIXED | the key map lacked the line commands (BB-UIX-010) | yes | `C1/D2/S06/BB_BUG_021_MOVE_COPY_AND_SELECT_LINE_KEYS_DO_NOTHING.md` |
| BB-BUG-022 | A click right of a line's end does nothing | FIXED | editor rows were only as wide as their text (BB-UIX-011) | yes | `C1/D2/S06/BB_BUG_022_A_CLICK_RIGHT_OF_A_LINES_END_DOES_NOTHING.md` |
| BB-BUG-023 | The sidebar header does not show which side is chosen | FIXED | design: a title and a link | yes | `C1/D2/S06/BB_BUG_023_THE_SIDEBAR_HEADER_DOES_NOT_SHOW_WHICH_SIDE_IS_CHOSEN.md` |
| BB-BUG-024 | Test groups do not collapse | FIXED | a fold marker with no fold (BB-UIX-012) | yes | `C1/D2/S06/BB_BUG_024_TEST_GROUPS_DO_NOT_COLLAPSE.md` |
| BB-BUG-025 | A click on a test does not open it | FIXED | design: not built | yes | `C1/D2/S07/BB_BUG_025_A_CLICK_ON_A_TEST_DOES_NOT_OPEN_IT.md` |
| BB-BUG-026 | A running test cannot be stopped | FIXED | the stop was far from the run (BB-UIX-013) | yes | `C1/D2/S07/BB_BUG_026_A_RUNNING_TEST_CANNOT_BE_STOPPED.md` |
| BB-BUG-027 | backback leaves temp folders behind | FIXED | temp folders outlive their process (BB-PRC-004) | yes | `C1/D2/S07/BB_BUG_027_BACKBACK_LEAVES_TEMP_FOLDERS_BEHIND.md` |
| BB-BUG-028 | Cmd+C does not copy | FIXED | design: not built | yes | `C1/D2/S07/BB_BUG_028_CMD_C_DOES_NOT_COPY.md` |
| BB-BUG-029 | The Explorer does not follow the active file | FIXED | design: not built | yes | `C1/D3/S08/BB_BUG_029_THE_EXPLORER_DOES_NOT_FOLLOW_THE_ACTIVE_FILE.md` |
| BB-BUG-030 | A binary file opens with a raw decoder error | FIXED | the decoder's text | yes | `C1/D3/S08/BB_BUG_030_A_BINARY_FILE_OPENS_WITH_A_RAW_DECODER_ERROR.md` |
| BB-BUG-031 | The Explorer seems to hide files | OBSERVED | not a bug: the folders hold only caches (BB-UIX-014) | no | `C1/D3/S08/BB_BUG_031_THE_EXPLORER_SEEMS_TO_HIDE_FILES.md` |
| BB-BUG-032 | A very long line freezes the editor | FIXED | every frame shaped and styled the whole line (BB-PRF-004) | yes | `C1/D3/S08/BB_BUG_032_A_VERY_LONG_LINE_FREEZES_THE_EDITOR.md` |
| BB-BUG-033 | The cursor stops inside a combined character | FIXED | moves stepped one code point, not one grapheme | yes | `C1/D3/S09/BB_BUG_033_THE_CURSOR_STOPS_INSIDE_A_COMBINED_CHARACTER.md` |
| BB-BUG-034 | A save overwrites another tool's change | FIXED | save never compared the file with what it had read (BB-UIX-015) | yes | `C1/D3/S09/BB_BUG_034_A_SAVE_OVERWRITES_ANOTHER_TOOLS_CHANGE.md` |
| BB-BUG-035 | A clean tab keeps stale text after an outside change | FIXED | nothing compared the file with what was read (BB-UIX-015) | yes | `C1/D3/S09/BB_BUG_035_A_CLEAN_TAB_KEEPS_STALE_TEXT_AFTER_AN_OUTSIDE_CHANGE.md` |
| BB-BUG-036 | Cmd+P does not open Go to File | FIXED | the key map lacked VS Code's keys (BB-UIX-010) | yes | `C1/D3/S09/BB_BUG_036_CMD_P_DOES_NOT_OPEN_GO_TO_FILE.md` |
| BB-BUG-037 | The Explorer misses files made or removed outside | FIXED | each folder was read once (BB-UIX-016) | yes | `C1/D3/S10/BB_BUG_037_THE_EXPLORER_MISSES_FILES_MADE_OR_REMOVED_OUTSIDE.md` |
| BB-BUG-038 | A second window wipes the first's Roslyn folder | FIXED | two workers shared one owned temp folder (BB-PRC-005) | yes | `C1/D3/S10/BB_BUG_038_A_SECOND_WINDOW_WIPES_THE_FIRSTS_ROSLYN_FOLDER.md` |
| BB-BUG-039 | No pinned tabs and no bulk close | FIXED | not built (VS Code tab menu) | yes | `C1/D3/S10/BB_BUG_039_NO_PINNED_TABS_AND_NO_BULK_CLOSE.md` |
| BB-BUG-040 | No split editor: a tab cannot be dragged to the right side | FIXED | not built (VS Code split editor) | yes | `C1/D3/S10/BB_BUG_040_NO_SPLIT_EDITOR.md` |
| BB-BUG-041 | The save prompt acts on the wrong tab after tabs move | FIXED | the prompt held an index, not a file | yes | `C1/D3/S11/BB_BUG_041_THE_SAVE_PROMPT_ACTS_ON_THE_WRONG_TAB_AFTER_A_MOVE.md` |
| BB-BUG-042 | Go to File / Find in Files cannot be closed by a click outside | FIXED | only Escape closed it, and only with key focus | yes | `C1/D3/S11/BB_BUG_042_GO_TO_FILE_AND_FIND_IN_FILES_CANNOT_BE_CLOSED_BY_A_CLICK.md` |
| BB-BUG-043 | Go to File still did not close on a click outside | FIXED | the fix was merged, not installed (BB-BLD-003) | yes | `C1/D3/S11/BB_BUG_043_GO_TO_FILE_STILL_DID_NOT_CLOSE_ON_A_CLICK_OUTSIDE.md` |
| BB-BUG-044 | The panes and the results panel could not be resized; long results were cut off | FIXED | not built (fixed sizes, clipped rows) | yes | `C1/D3/S11/BB_BUG_044_PANES_AND_THE_RESULTS_PANEL_COULD_NOT_BE_RESIZED_OR_SCROLLED.md` |
| BB-BUG-045 | Only two editor panes; three files could not show side by side | FIXED | not built (two groups only) | yes | `C1/D4/S12/BB_BUG_045_ONLY_TWO_EDITOR_PANES.md` |
| BB-BUG-046 | The terminal's text was unreadable and its height fixed | FIXED | no text colour set (gpui default black); fixed height | yes | `C1/D4/S12/BB_BUG_046_TERMINAL_TEXT_UNREADABLE_AND_HEIGHT_FIXED.md` |
| BB-BUG-047 | Panels had no close button; the View menu had no Terminal | FIXED | not built (key-only close; no menu item) | yes | `C1/D4/S12/BB_BUG_047_PANELS_COULD_NOT_BE_CLOSED_AND_NO_TERMINAL_MENU.md` |
| BB-BUG-048 | No New Window; open windows were not in the Window menu | FIXED | not built (untitled windows; no New Window) | yes | `C1/D4/S12/BB_BUG_048_NO_NEW_WINDOW_AND_WINDOWS_NOT_IN_THE_WINDOW_MENU.md` |
| BB-BUG-049 | A wider window kept the text scrolled sideways and cut off | FIXED | scroll clamped only on a swipe | yes | `C1/D4/S13/BB_BUG_049_A_WIDER_WINDOW_KEPT_THE_TEXT_SCROLLED_AND_CUT.md` |
| BB-BUG-050 | No New File, New Folder, Rename or Delete in the Explorer | FIXED | not built | yes | `C1/D4/S13/BB_BUG_050_NO_NEW_RENAME_OR_DELETE_IN_THE_EXPLORER.md` |
| BB-BUG-051 | The editor had no scroll bars | FIXED | not built | yes | `C1/D4/S13/BB_BUG_051_NO_SCROLL_BARS_IN_THE_EDITOR.md` |
| BB-BUG-052 | No Copy, Paste or Open Terminal Here in the Explorer | FIXED | not built | yes | `C1/D4/S13/BB_BUG_052_NO_COPY_PASTE_OR_TERMINAL_HERE_IN_THE_EXPLORER.md` |
| BB-BUG-053 | Terminal text could not be selected | FIXED | not built | yes | `C1/D4/S14/BB_BUG_053_TERMINAL_TEXT_COULD_NOT_BE_SELECTED.md` |
| BB-BUG-054 | Selected text was unreadable | FIXED | accent as selection bg (1.29:1) | yes | `C1/D4/S14/BB_BUG_054_SELECTED_TEXT_WAS_UNREADABLE.md` |
| BB-BUG-055 | The view did not follow the cursor | FIXED | no scroll on cursor keys | yes | `C1/D4/S14/BB_BUG_055_THE_VIEW_DID_NOT_FOLLOW_THE_CURSOR.md` |
| BB-BUG-056 | The line end hid under a right-docked panel | FIXED | text width ignored the right dock | yes | `C1/D4/S14/BB_BUG_056_THE_LINE_END_HID_UNDER_A_RIGHT_DOCKED_PANEL.md` |
| BB-BUG-057 | The find box took no paste and had no caret | FIXED | box kept a plain string, Cmd keys dropped | yes | `C1/D4/S15/BB_BUG_057_THE_FIND_BOX_TOOK_NO_PASTE_AND_HAD_NO_CARET.md` |
| BB-BUG-058 | The terminal could not scroll back | FIXED | the panel never handled the wheel | yes | `C1/D4/S15/BB_BUG_058_THE_TERMINAL_COULD_NOT_SCROLL_BACK.md` |
| BB-BUG-059 | Deleted folders took the recent list's places | FIXED | the saved list kept gone folders | yes | `C1/D4/S15/BB_BUG_059_DELETED_FOLDERS_TOOK_THE_RECENT_LIST_PLACES.md` |
| BB-BUG-060 | The tree did not come back to the file | FIXED | followed a file once per new tab only | yes | `C1/D4/S15/BB_BUG_060_THE_TREE_DID_NOT_COME_BACK_TO_THE_FILE.md` |
| BB-BUG-061 | An Apex project listed through an Apex CLI that cannot list | FIXED | app picked Apex by package, not by `apexTestkit.*` keys | yes | `C2/D5/S16/BB_BUG_061_AN_APEX_PROJECT_LISTED_THROUGH_AN_APEX_CLI_THAT_CANNOT_LIST.md` |
| BB-BUG-062 | The tab bar hid tabs past its edge | FIXED | bar was `overflow_hidden`, no scroll, no reveal | yes | `C2/D5/S16/BB_BUG_062_THE_TAB_BAR_HID_TABS_PAST_ITS_EDGE.md` |
| BB-BUG-063 | Dragging the pane divider selected the text under it | FIXED | the line's drag handler never asked where the press began | yes | `C2/D5/S16/BB_BUG_063_DRAGGING_THE_PANE_DIVIDER_SELECTED_THE_TEXT_UNDER_IT.md` |
| BB-BUG-064 | Debug on a test was a placeholder that never debugged | FIXED | Debug never wired to the debugger; no attach | yes | `C2/D5/S16/BB_BUG_064_DEBUG_ON_A_TEST_WAS_A_PLACEHOLDER_THAT_NEVER_DEBUGGED.md` |
| BB-BUG-065 | The gutter's ▶ never ran its test | FIXED | full name compared with the declared name | yes | `C2/D5/S17/BB_BUG_065_THE_GUTTER_RUN_MARK_NEVER_RAN_ITS_TEST.md` |
| BB-BUG-066 | A debugged test never got its ✓ or ✕ | FIXED | the session stopped the run without reading its results | yes | `C2/D5/S17/BB_BUG_066_A_DEBUGGED_TEST_NEVER_GOT_ITS_VERDICT.md` |
| BB-BUG-067 | Back did not return from a debugger step into another file | FIXED | a stop's jump was not recorded; keys only in the editor | yes | `C2/D5/S17/BB_BUG_067_BACK_DID_NOT_RETURN_FROM_A_DEBUGGER_STEP.md` |
| BB-BUG-068 | Dragging the execution arrow was refused | FIXED | stock netcoredbg installed; its real test was vacuous | yes | `C2/D5/S17/BB_BUG_068_DRAGGING_THE_EXECUTION_ARROW_WAS_REFUSED.md` |
| BB-BUG-069 | A click in the Find box fell through to the editor | FIXED | overlays did not occlude the editor under them | yes | `C2/D5/S18/BB_BUG_069_A_CLICK_IN_THE_FIND_BOX_FELL_THROUGH_TO_THE_EDITOR.md` |
| BB-BUG-070 | The title bar zoomed twice on a double click | FIXED | every press started the system drag | yes | `C2/D5/S18/BB_BUG_070_THE_TITLE_BAR_ZOOMED_TWICE_ON_A_DOUBLE_CLICK.md` |
| BB-BUG-071 | The arrow could not be dragged up in an async spec | OBSERVED | targets only on a statement's first line; one-line refusal | yes | `C2/D5/S18/BB_BUG_071_THE_ARROW_COULD_NOT_BE_DRAGGED_UP_IN_AN_ASYNC_SPEC.md` |
| BB-BUG-072 | The hover showed a type for code that was not running | FIXED | the running code is the session's build | yes | `C2/D5/S18/BB_BUG_072_THE_HOVER_SHOWED_A_TYPE_FOR_CODE_THAT_WAS_NOT_RUNNING.md` |
| BB-BUG-073 | VARIABLES did not say the file changed since the build | FIXED | same as BB-BUG-072: `x` was typed after the build | yes | `C2/D5/S19/BB_BUG_073_VARIABLES_DID_NOT_SAY_THE_FILE_CHANGED_SINCE_THE_BUILD.md` |
| BB-BUG-074 | The editor's test menu opened below the test | FIXED | nearest test above, with no end | yes | `C2/D5/S19/BB_BUG_074_THE_EDITORS_TEST_MENU_OPENED_BELOW_THE_TEST.md` |
| BB-BUG-075 | A Rust or Python tab was sent to Roslyn as C# | FIXED | one link for every file | yes | `C2/D5/S19/BB_BUG_075_A_RUST_OR_PYTHON_TAB_WAS_SENT_TO_ROSLYN_AS_CSHARP.md` |
| BB-BUG-076 | Hover or F12 in a Rust file failed with content modified | FIXED | content modified shown as a failure | yes | `C2/D5/S19/BB_BUG_076_HOVER_OR_F12_IN_A_RUST_FILE_FAILED_WITH_CONTENT_MODIFIED.md` |
| BB-BUG-077 | A failed Rust or Python request said C# server | FIXED | Roslyn's name in the failure text | yes | `C2/D6/S20/BB_BUG_077_A_FAILED_RUST_OR_PYTHON_REQUEST_SAID_CSHARP_SERVER.md` |
| BB-BUG-078 | A Dock start without a shell PATH missed rust-analyzer | FIXED | fallback PATH without ~/.cargo/bin | yes | `C2/D6/S20/BB_BUG_078_A_DOCK_START_WITHOUT_A_SHELL_PATH_MISSED_RUST_ANALYZER.md` |
| BB-BUG-079 | An empty rename said Renamed: 0 edits | FIXED | empty plan opened a preview | yes | `C2/D6/S20/BB_BUG_079_AN_EMPTY_RENAME_SAID_RENAMED_0_EDITS.md` |
| BB-BUG-080 | Back stayed live and did nothing after its file's tab closed | FIXED | BB-UIX-017 | yes | `C2/D6/S20/BB_BUG_080_BACK_STAYED_LIVE_AND_DID_NOTHING_AFTER_ITS_FILES_TAB_CLOSED.md` |
| BB-BUG-081 | A row into a deleted file named no file and left a jump waiting | FIXED | BB-UIX-018 | yes | `C2/D6/S21/BB_BUG_081_A_ROW_INTO_A_DELETED_FILE_NAMED_NO_FILE_AND_LEFT_A_JUMP_WAITING.md` |
| BB-BUG-082 | A Debug start sent the breakpoints of a deleted file | FIXED | BB-UIX-019 | yes | `C2/D6/S21/BB_BUG_082_A_DEBUG_START_SENT_THE_BREAKPOINTS_OF_A_DELETED_FILE.md` |
| BB-BUG-083 | Go to a deleted local file said "not on this machine" | FIXED | BB-UIX-020 | yes | `C2/D6/S21/BB_BUG_083_GO_TO_A_DELETED_LOCAL_FILE_SAID_NOT_ON_THIS_MACHINE.md` |
| BB-BUG-084 | A Rust or Python file argument opened its own folder | FIXED | BB-UIX-021 | yes | `C2/D6/S21/BB_BUG_084_A_RUST_OR_PYTHON_FILE_ARGUMENT_OPENED_ITS_OWN_FOLDER.md` |
| BB-BUG-085 | Every tab showed a C# mark | FIXED | BB-UIX-022 | yes | `C2/D6/S22/BB_BUG_085_EVERY_TAB_SHOWED_A_CSHARP_MARK.md` |
| BB-BUG-086 | The C# and Debug menus showed without C# | FIXED | BB-UIX-022 | yes | `C2/D6/S22/BB_BUG_086_THE_CSHARP_AND_DEBUG_MENUS_SHOWED_WITHOUT_CSHARP.md` |
| BB-BUG-087 | Start Page RECENT sat apart from its list and keys did not scroll | FIXED | BB-UIX-023 | yes | `C2/D6/S22/BB_BUG_087_START_PAGE_RECENT_SAT_APART_FROM_ITS_LIST_AND_KEYS_DID_NOT_SCROLL.md` |
| BB-BUG-088 | A broken Cargo.toml filled the Explorer with cargo's error | FIXED | BB-UIX-024 | yes | `C2/D6/S22/BB_BUG_088_A_BROKEN_CARGO_TOML_FILLED_THE_EXPLORER_WITH_CARGOS_ERROR.md` |
| BB-BUG-089 | A scratch folder showed in RECENT | OBSERVED | not a bug: RECENT lists every workspace opened; it still existed | no | `C2/D6/S23/BB_BUG_089_A_SCRATCH_FOLDER_SHOWED_IN_RECENT.md` |
| BB-BUG-090 | A file of no language reached the C# server | FIXED | BB-UIX-025 | yes | `C2/D6/S23/BB_BUG_090_A_FILE_OF_NO_LANGUAGE_REACHED_THE_C_SERVER.md` |
| BB-BUG-091 | Find in Files offered no Project in a Rust or Python window | FIXED | BB-UIX-025 | yes | `C2/D6/S23/BB_BUG_091_FIND_IN_FILES_OFFERED_NO_PROJECT_IN_A_RUST_OR_PYTHON_WINDOW.md` |
| BB-BUG-092 | A rename and Cmd+N opened C# tabs in a Rust window | FIXED | BB-UIX-025 | yes | `C2/D6/S23/BB_BUG_092_A_RENAME_AND_CMD_N_OPENED_C_TABS_IN_A_RUST_WINDOW.md` |
