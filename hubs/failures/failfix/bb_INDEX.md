# Failfix index

| Id | Status | Fix | Path |
|---|---|---|---|
| BB-BLD-002 | FIXED | GUI binary is the bundle executable; served app verified | `SHIP/BLD/BB_BLD_002_BUNDLE_OPENED_NO_WINDOW.md` |
| BB-LSP-001 | FIXED | Count projects by distinct path; wait on readiness | `APP/LSP/S01/BB_LSP_001_READINESS_WAIT_ON_A_FIXED_WALL.md` |
| BB-PRF-001 | FIXED | Files over 4 MiB parse ±1,000 lines around the view | `SYS/PRF/BB_PRF_001_WHOLE_FILE_SYNTAX_TREE_HOLDS_THE_HEAP.md` |
| BB-UIX-001 | FIXED | One shaped line per row: caret quad, clicks, popups and hover share `Columns` | `APP/UIX/S01/BB_UIX_001_CURSOR_DRAWN_AS_A_TEXT_CHARACTER.md` |
| BB-LSP-002 | FIXED | `first_diagnostics` skips late Progress, names any other event | `APP/LSP/BB_LSP_002_TEST_TAKES_THE_FIRST_EVENT_AFTER_OPEN.md` |
| BB-CIP-001 | FIXED | CI installs pinned dotnet, Roslyn server and netcoredbg; `--no-fail-fast` | `SHIP/CIP/BB_CIP_001_CI_RUNNER_HAS_NO_ROSLYN_SERVER.md` |
| BB-CIP-002 | FIXED | `runs-on: macos-latest` | `SHIP/CIP/BB_CIP_002_RUNNER_LABEL_NEVER_GETS_A_MACHINE.md` |
| BB-UIX-002 | FIXED | Minimize/Zoom handled on the root view with its own Window | `APP/UIX/S01/BB_UIX_002_MINIMIZE_ON_THE_ACTIVE_WINDOW.md` |
| BB-PRC-001 | FIXED | Own every child's process group; every exit ends the registry | `SYS/PRC/BB_PRC_001_A_CHILD_GROUP_OUTLIVES_ITS_OWNER.md` |
| BB-UIX-003 | FIXED | Gutter width scales with the size | `APP/UIX/S01/BB_UIX_003_GUTTER_WIDTH_FIXED_WHEN_ZOOMED.md` |
| BB-PRC-002 | FIXED | Spawn on the UI thread, wait in the background | `SYS/PRC/BB_PRC_002_BUILD_SPAWNED_FROM_A_BACKGROUND_THREAD_HANGS.md` |
| BB-BLD-001 | FIXED | `install.sh` from clean `master` to `/Applications`; `BBCommit` stamp | `SHIP/BLD/BB_BLD_001_INSTALL_FROM_MASTER.md` |
| BB-LSP-004 | FIXED | `bb open --restore` waits for a restore end line before shutdown | `APP/LSP/S01/BB_LSP_004_RESTORE_LINE_AFTER_OPEN_EXITS.md` |
| BB-PRF-002 | FIXED | Box opens at once; files list on the background executor | `SYS/PRF/BB_PRF_002_GO_TO_FILE_WALKS_THE_ROOT_ON_THE_UI_THREAD.md` |
| BB-LSP-005 | FIXED | Assert the server reply to our own request (shutdown), not a side-effect line | `APP/LSP/S02/BB_LSP_005_TEST_ASSERTED_AN_INCIDENTAL_ROSLYN_LOG_LINE.md` |
| BB-CIP-003 | FIXED | `cargo update -p async-executor` 1.14.0 -> 1.14.1 | `SHIP/CIP/BB_CIP_003_A_YANKED_TRANSITIVE_CRATE_FAILS_CARGO_DENY.md` |
| BB-GAT-001 | FIXED | Assert the parse contract: extent always, exact tree on valid text, converge after undo | `SHIP/GAT/BB_GAT_001_A_PROPERTY_TEST_ASKED_MORE_THAN_TREE_SITTER_PROMISES.md` |
| BB-DAP-001 | FIXED | Headline from `details.fullTypeName` and `details.message`; else the id without its category | `APP/DAP/BB_DAP_001_EXCEPTION_INFO_NAMES_THE_CATEGORY_AND_MODULE.md` |
| BB-LSP-006 | FIXED | `Loaded::restore_ended`; `bb open --restore` waits only when no end line came during the load | `APP/LSP/S02/BB_LSP_006_A_RESTORE_THAT_ENDS_DURING_THE_LOAD_IS_WAITED_FOR_AGAIN.md` |
| BB-ENV-001 | FIXED | Adopt the login shell's PATH when there is no TERM; find installed tools in their own folders | `SYS/ENV/BB_ENV_001_AN_APP_LAUNCHED_BY_LAUNCHD_HAS_NO_SHELL_PATH.md` |
| BB-UIX-004 | FIXED | `\r\n` is one line break in every move and edit | `APP/UIX/S01/BB_UIX_004_A_CRLF_LINE_BREAK_COUNTED_AS_TWO_CURSOR_STOPS.md` |
| BB-UIX-005 | FIXED | The key map reads Option and Ctrl first: word moves and deletes, the Emacs line keys | `APP/UIX/S02/BB_UIX_005_THE_KEY_MAP_IGNORED_OPTION_AND_CTRL.md` |
| BB-UIX-006 | FIXED | A per-tab `scroll_x`: clipped text viewport, caret/clicks offset, wheel scroll, caret kept in sight | `APP/UIX/S02/BB_UIX_006_ROWS_HAD_NO_HORIZONTAL_OFFSET.md` |
| BB-UIX-007 | FIXED | Line-wise commands in the text model; Tab, Shift+Tab, Cmd+Backspace, Cmd+Delete get their own arms | `APP/UIX/S02/BB_UIX_007_TAB_AND_THE_CMD_DELETES_HAD_NO_LINE_COMMANDS.md` |
| BB-UIX-008 | FIXED | `select_all` and `toggle_line_comment` in the text model, arms in `command_key` | `APP/UIX/S02/BB_UIX_008_THE_CMD_MAP_LACKED_SELECT_ALL_AND_THE_COMMENT_TOGGLE.md` |
| BB-UIX-009 | FIXED | `RenameInput::fresh`: the old name opens selected; the first key replaces it | `APP/UIX/S03/BB_UIX_009_THE_RENAME_BOX_DID_NOT_SELECT_THE_OLD_NAME.md` |
| BB-LSP-007 | FIXED | Every change and every refresh marks every open document; refresh support declared | `APP/LSP/S02/BB_LSP_007_AN_EDIT_PULLED_AGAIN_ONLY_ITS_OWN_FILE.md` |
| BB-UIX-010 | FIXED | Line commands in the text model, arms in `document_command` and `text_key`; move and copy since BB-BUG-021 | `APP/UIX/S03/BB_UIX_010_THE_KEY_MAP_LACKED_VS_CODES_LINE_COMMANDS.md` |
| BB-UIX-011 | FIXED | `w_full` on the editor row | `APP/UIX/S03/BB_UIX_011_EDITOR_ROWS_WERE_ONLY_AS_WIDE_AS_THEIR_TEXT.md` |
| BB-UIX-012 | FIXED | Heads are rows with a `Group` key, marker from state | `APP/UIX/S03/BB_UIX_012_A_FOLD_MARKER_WITH_NO_FOLD.md` |
| BB-UIX-013 | FIXED | The starting control turns into its stop while running | `APP/UIX/S04/BB_UIX_013_THE_STOP_WAS_FAR_FROM_THE_RUN.md` |
| BB-PRC-004 | FIXED | One owner per folder; sweep what a dead owner left | `SYS/PRC/BB_PRC_004_TEMP_FOLDERS_OUTLIVE_THEIR_PROCESS.md` |
| BB-PRF-003 | FIXED | `DOTNET_gcServer=0` for the Roslyn child unless the user set it | `SYS/PRF/BB_PRF_003_ROSLYN_RAN_THE_SERVER_GC.md` |
| BB-PRF-004 | FIXED | `RENDER_CAP` where a line is shaped and styled | `SYS/PRF/BB_PRF_004_EVERY_FRAME_SHAPED_AND_STYLED_THE_WHOLE_LINE.md` |
