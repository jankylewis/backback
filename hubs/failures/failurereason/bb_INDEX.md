# Failurereason index

One file per failure mechanism. This hub owns the id. Entries: 35 — FIXED 32, OBSERVED 2, OPEN 1.

| Id | Failure reason | Status | fix | lesson | toolgap | Path |
|---|---|---|---|---|---|---|
| BB-BLD-001 | An app built before a fix keeps running from a worktree bundle | FIXED | yes | yes | none | `SHIP/BLD/BB_BLD_001_STALE_APP_RUN_FROM_A_WORKTREE.md` |
| BB-BLD-002 | The `.app` opened no window; a first-frame readback said it did | FIXED | yes | yes | none | `SHIP/BLD/BB_BLD_002_BUNDLE_OPENED_NO_WINDOW.md` |
| BB-CIP-001 | The CI runner has no roslyn-language-server or netcoredbg | FIXED | yes | yes | BB-TG-004 | `SHIP/CIP/BB_CIP_001_CI_RUNNER_HAS_NO_ROSLYN_SERVER.md` |
| BB-LSP-001 | Roslyn tests wait on a fixed wall and count reloads twice | FIXED | yes | yes | none | `APP/LSP/S01/BB_LSP_001_READINESS_WAIT_ON_A_FIXED_WALL.md` |
| BB-PRF-001 | A whole-file syntax tree held 97% of the heap on a 20 MB file | FIXED | yes | yes | none | `SYS/PRF/BB_PRF_001_WHOLE_FILE_SYNTAX_TREE_HOLDS_THE_HEAP.md` |
| BB-PRF-002 | Go to File walked the whole root on the UI thread | FIXED | yes | yes | none | `SYS/PRF/BB_PRF_002_GO_TO_FILE_WALKS_THE_ROOT_ON_THE_UI_THREAD.md` |
| BB-UIX-001 | The cursor is drawn as a text character inside the line | FIXED | yes | yes | BB-TG-003 | `APP/UIX/S01/BB_UIX_001_CURSOR_DRAWN_AS_A_TEXT_CHARACTER.md` |
| BB-LSP-002 | A test took the first event after Open as the diagnostics | FIXED | yes | no | none | `APP/LSP/S01/BB_LSP_002_TEST_TAKES_THE_FIRST_EVENT_AFTER_OPEN.md` |
| BB-CIP-002 | A runner label no hosted machine serves keeps the job queued | FIXED | yes | no | none | `SHIP/CIP/BB_CIP_002_RUNNER_LABEL_NEVER_GETS_A_MACHINE.md` |
| BB-LSP-003 | Problems still counted a deleted error 6 s after the edit | OPEN | no | no | none | `APP/LSP/S01/BB_LSP_003_DIAGNOSTICS_LAG_AFTER_A_DELETE.md` |
| BB-UIX-002 | Minimize acted on `cx.active_window()`, None while another app is active | FIXED | yes | yes | BB-TG-005 | `APP/UIX/S01/BB_UIX_002_MINIMIZE_ON_THE_ACTIVE_WINDOW.md` |
| BB-PRC-001 | A child process group outlives its owner | FIXED | yes | yes | none | `SYS/PRC/BB_PRC_001_A_CHILD_GROUP_OUTLIVES_ITS_OWNER.md` |
| BB-LSP-004 | `open --restore` can exit before Roslyn logs the restore | FIXED | yes | no | none | `APP/LSP/S01/BB_LSP_004_RESTORE_LINE_AFTER_OPEN_EXITS.md` |
| BB-UIX-003 | The gutter kept 50 px when zoomed, so line numbers ran into the code | FIXED | yes | no | none | `APP/UIX/S01/BB_UIX_003_GUTTER_WIDTH_FIXED_WHEN_ZOOMED.md` |
| BB-PRC-002 | `dotnet build` spawned from a background thread never finished | FIXED | yes | yes | none | `SYS/PRC/BB_PRC_002_BUILD_SPAWNED_FROM_A_BACKGROUND_THREAD_HANGS.md` |
| BB-PRC-003 | `SN` Roslyn came from a niced readback launch, not the app | OBSERVED | no | no | BB-TG-006 | `SYS/PRC/BB_PRC_003_TOOLS_SPAWNED_FROM_BACKGROUND_THREADS.md` |
| BB-LSP-005 | A test asserted a Roslyn log line that only a side effect writes | FIXED | yes | yes | none | `APP/LSP/S02/BB_LSP_005_TEST_ASSERTED_AN_INCIDENTAL_ROSLYN_LOG_LINE.md` |
| BB-CIP-003 | A yanked transitive crate fails `cargo deny` on CI | FIXED | yes | yes | none | `SHIP/CIP/BB_CIP_003_A_YANKED_TRANSITIVE_CRATE_FAILS_CARGO_DENY.md` |
| BB-GAT-001 | A property test asked more than tree-sitter promises | FIXED | yes | yes | none | `SHIP/GAT/BB_GAT_001_A_PROPERTY_TEST_ASKED_MORE_THAN_TREE_SITTER_PROMISES.md` |
| BB-DAP-001 | netcoredbg's `exceptionInfo` names the CLR category and the module, not the type and message | FIXED | yes | yes | none | `APP/DAP/BB_DAP_001_EXCEPTION_INFO_NAMES_THE_CATEGORY_AND_MODULE.md` |
| BB-LSP-006 | A restore that ends during the load is waited for again, 120 s, then `restored : false` | FIXED | yes | yes | none | `APP/LSP/S02/BB_LSP_006_A_RESTORE_THAT_ENDS_DURING_THE_LOAD_IS_WAITED_FOR_AGAIN.md` |
| BB-ENV-001 | An app launched by launchd has no shell PATH, so dotnet and netcoredbg are not found | FIXED | yes | yes | BB-TG-012 | `SYS/ENV/BB_ENV_001_AN_APP_LAUNCHED_BY_LAUNCHD_HAS_NO_SHELL_PATH.md` |
| BB-UIX-004 | A CRLF line break counted as two cursor stops | FIXED | yes | yes | none | `APP/UIX/S01/BB_UIX_004_A_CRLF_LINE_BREAK_COUNTED_AS_TWO_CURSOR_STOPS.md` |
| BB-UIX-005 | The key map ignored Option and Ctrl | FIXED | yes | yes | none | `APP/UIX/S02/BB_UIX_005_THE_KEY_MAP_IGNORED_OPTION_AND_CTRL.md` |
| BB-UIX-006 | Editor rows had no horizontal offset, so the list clipped every wide line | FIXED | yes | yes | none | `APP/UIX/S02/BB_UIX_006_ROWS_HAD_NO_HORIZONTAL_OFFSET.md` |
| BB-UIX-007 | Tab and the Cmd deletes had no line-wise commands: Tab typed over the selection, Cmd+Backspace did nothing | FIXED | yes | yes | none | `APP/UIX/S02/BB_UIX_007_TAB_AND_THE_CMD_DELETES_HAD_NO_LINE_COMMANDS.md` |
| BB-UIX-008 | The Cmd map lacked Select All and the comment toggle | FIXED | yes | yes | none | `APP/UIX/S02/BB_UIX_008_THE_CMD_MAP_LACKED_SELECT_ALL_AND_THE_COMMENT_TOGGLE.md` |
| BB-UIX-009 | The rename box did not select the old name, so a typed name was glued onto it | FIXED | yes | yes | none | `APP/UIX/S03/BB_UIX_009_THE_RENAME_BOX_DID_NOT_SELECT_THE_OLD_NAME.md` |
| BB-LSP-007 | An edit pulled diagnostics again only for its own file, so other open files kept lists of the old text | FIXED | yes | yes | none | `APP/LSP/S02/BB_LSP_007_AN_EDIT_PULLED_AGAIN_ONLY_ITS_OWN_FILE.md` |
| BB-UIX-010 | The key map lacked VS Code's line commands | FIXED | yes | yes | none | `APP/UIX/S03/BB_UIX_010_THE_KEY_MAP_LACKED_VS_CODES_LINE_COMMANDS.md` |
| BB-UIX-011 | Editor rows were only as wide as their text, so a click past the text hit nothing | FIXED | yes | yes | none | `APP/UIX/S03/BB_UIX_011_EDITOR_ROWS_WERE_ONLY_AS_WIDE_AS_THEIR_TEXT.md` |
| BB-UIX-012 | A ▾ written into a label promised a fold the row could not do | FIXED | yes | yes | none | `APP/UIX/S03/BB_UIX_012_A_FOLD_MARKER_WITH_NO_FOLD.md` |
| BB-UIX-013 | The only stop sat in the toolbar, far from the ▶ that started the run | FIXED | yes | yes | none | `APP/UIX/S04/BB_UIX_013_THE_STOP_WAS_FAR_FROM_THE_RUN.md` |
| BB-PRC-004 | Temp folders named by pid were never removed: a static or a quit never drops | FIXED | yes | yes | none | `SYS/PRC/BB_PRC_004_TEMP_FOLDERS_OUTLIVE_THEIR_PROCESS.md` |
| BB-UIX-014 | A folder with only `__pycache__` reads as a tree that hides files | OBSERVED | no | yes | none | `APP/UIX/S04/BB_UIX_014_A_FOLDER_OF_ONLY_CACHES_READS_AS_A_TREE_THAT_HIDES_FILES.md` |
