# Lessonlearn index

| Id | Status | Lesson | Path |
|---|---|---|---|
| BB-BLD-001 | OBSERVED | Name the build before reading a UI report | `SHIP/BLD/BB_BLD_001_STALE_APP_RUN_FROM_A_WORKTREE.md` |
| BB-BLD-002 | FIXED | A log line is not a window; open the served app | `SHIP/BLD/BB_BLD_002_BUNDLE_OPENED_NO_WINDOW.md` |
| BB-LSP-001 | FIXED | Wait on a condition, never a wall; a flake is a finding | `APP/LSP/S01/BB_LSP_001_READINESS_WAIT_ON_A_FIXED_WALL.md` |
| BB-PRF-001 | FIXED | Profile who holds the bytes before changing code | `SYS/PRF/S01/BB_PRF_001_WHOLE_FILE_SYNTAX_TREE_HOLDS_THE_HEAP.md` |
| BB-PRF-002 | FIXED | Measure every feature at its own declared limit | `SYS/PRF/S01/BB_PRF_002_GO_TO_FILE_WALKS_THE_ROOT_ON_THE_UI_THREAD.md` |
| BB-UIX-001 | FIXED | Nothing outside the file goes into the text; every x comes from the shaped line | `APP/UIX/D1/S01/BB_UIX_001_CURSOR_DRAWN_AS_A_TEXT_CHARACTER.md` |
| BB-CIP-001 | FIXED | CI installs every external tool, pinned; `--no-fail-fast` | `SHIP/CIP/BB_CIP_001_CI_RUNNER_HAS_NO_ROSLYN_SERVER.md` |
| BB-UIX-002 | FIXED | A window command acts on the window the event reached | `APP/UIX/D1/S01/BB_UIX_002_MINIMIZE_ON_THE_ACTIVE_WINDOW.md` |
| BB-PRC-001 | FIXED | Own the group, not the pid | `SYS/PRC/S01/BB_PRC_001_A_CHILD_GROUP_OUTLIVES_ITS_OWNER.md` |
| BB-PRC-002 | FIXED | Spawn tools on a normal-QoS thread | `SYS/PRC/S01/BB_PRC_002_BUILD_SPAWNED_FROM_A_BACKGROUND_THREAD_HANGS.md` |
| BB-LSP-005 | FIXED | Assert only what the server writes because of our request | `APP/LSP/S02/BB_LSP_005_TEST_ASSERTED_AN_INCIDENTAL_ROSLYN_LOG_LINE.md` |
| BB-CIP-003 | FIXED | A yank lives in the index; ask crates.io when CI and local deny disagree | `SHIP/CIP/BB_CIP_003_A_YANKED_TRANSITIVE_CRATE_FAILS_CARGO_DENY.md` |
| BB-CIP-004 | FIXED | Gate a merge on structured check fields, never grepped text | `SHIP/CIP/BB_CIP_004_A_CHECK_NO_RUNNER_TOOK_WAS_MERGED_OVER.md` |
| BB-CIP-005 | FIXED | Test the value, not the exit code, before removing work | `SHIP/CIP/BB_CIP_005_CLEANUP_RAN_ON_AN_OPEN_PR.md` |
| BB-GAT-001 | FIXED | Assert the library's contract, not a stronger one | `SHIP/GAT/S01/BB_GAT_001_A_PROPERTY_TEST_ASKED_MORE_THAN_TREE_SITTER_PROMISES.md` |
| BB-GAT-002 | FIXED | A test's scratch folder is its process's own, never a fixed path | `SHIP/GAT/S01/BB_GAT_002_A_FIXTURE_FOLDER_SHARED_BY_PARALLEL_TEST_PROCESSES.md` |
| BB-GAT-003 | FIXED | Wait for the state, never a fixed time | `SHIP/GAT/S01/BB_GAT_003_A_FIXED_SLEEP_WAS_TOO_SHORT_ON_A_LOADED_RUNNER.md` |
| BB-GAT-004 | FIXED | Menu or key changes run the launch tests first | `SHIP/GAT/S01/BB_GAT_004_A_MENU_ITEM_ADDED_WITHOUT_THE_FULL_SUITE.md` |
| BB-DAP-001 | FIXED | Print the adapter's real reply before shaping text a user reads | `APP/DAP/BB_DAP_001_EXCEPTION_INFO_NAMES_THE_CATEGORY_AND_MODULE.md` |
| BB-LSP-006 | FIXED | Two waits on one stream share one reader state | `APP/LSP/S02/BB_LSP_006_A_RESTORE_THAT_ENDS_DURING_THE_LOAD_IS_WAITED_FOR_AGAIN.md` |
| BB-ENV-001 | FIXED | Read back a macOS app the way the Dock starts it: env -i, launchd's PATH, no TERM | `SYS/ENV/BB_ENV_001_AN_APP_LAUNCHED_BY_LAUNCHD_HAS_NO_SHELL_PATH.md` |
| BB-UIX-004 | FIXED | The unit of movement is what the user sees as one thing; test on CRLF too | `APP/UIX/D1/S01/BB_UIX_004_A_CRLF_LINE_BREAK_COUNTED_AS_TWO_CURSOR_STOPS.md` |
| BB-UIX-005 | FIXED | Read a key with its modifiers; sweep the whole modifier table | `APP/UIX/D1/S02/BB_UIX_005_THE_KEY_MAP_IGNORED_OPTION_AND_CTRL.md` |
| BB-UIX-006 | FIXED | Read an editor back to the far edges of its text, by key and by pointer | `APP/UIX/D1/S02/BB_UIX_006_ROWS_HAD_NO_HORIZONTAL_OFFSET.md` |
| BB-UIX-007 | FIXED | A key with a selection is its own case; the default insert arm is for printable chars only | `APP/UIX/D1/S02/BB_UIX_007_TAB_AND_THE_CMD_DELETES_HAD_NO_LINE_COMMANDS.md` |
| BB-UIX-008 | FIXED | The reference editor's key table is the checklist; a missing arm is a bug found by the sweep | `APP/UIX/D1/S02/BB_UIX_008_THE_CMD_MAP_LACKED_SELECT_ALL_AND_THE_COMMENT_TOGGLE.md` |
| BB-UIX-009 | FIXED | A rig presses no helper key a user would not; a needed helper key is the bug | `APP/UIX/D1/S03/BB_UIX_009_THE_RENAME_BOX_DID_NOT_SELECT_THE_OLD_NAME.md` |
| BB-LSP-007 | FIXED | Ask again for a list whenever what it is computed from changes | `APP/LSP/S02/BB_LSP_007_AN_EDIT_PULLED_AGAIN_ONLY_ITS_OWN_FILE.md` |
| BB-UIX-010 | FIXED | Sweep the reference keymap page by page; one page swept says nothing about the next | `APP/UIX/D1/S03/BB_UIX_010_THE_KEY_MAP_LACKED_VS_CODES_LINE_COMMANDS.md` |
| BB-UIX-011 | FIXED | An element that takes clicks spans the area a user can click | `APP/UIX/D1/S03/BB_UIX_011_EDITOR_ROWS_WERE_ONLY_AS_WIDE_AS_THEIR_TEXT.md` |
| BB-UIX-012 | FIXED | A marker that promises an action comes from state and has a handler | `APP/UIX/D1/S03/BB_UIX_012_A_FOLD_MARKER_WITH_NO_FOLD.md` |
| BB-UIX-013 | FIXED | The way to stop an action appears where it was started, only while running | `APP/UIX/D2/S04/BB_UIX_013_THE_STOP_WAS_FAR_FROM_THE_RUN.md` |
| BB-PRC-004 | FIXED | Every resource has an owner and a sweep; audit the disk too | `SYS/PRC/S01/BB_PRC_004_TEMP_FOLDERS_OUTLIVE_THEIR_PROCESS.md` |
| BB-UIX-014 | OBSERVED | Diff the view against `ls -A` before fixing a missing file | `APP/UIX/D2/S04/BB_UIX_014_A_FOLDER_OF_ONLY_CACHES_READS_AS_A_TREE_THAT_HIDES_FILES.md` |
| BB-PRF-003 | FIXED | Count every child's memory; a child's runtime defaults are ours to choose | `SYS/PRF/S01/BB_PRF_003_ROSLYN_RAN_THE_SERVER_GC.md` |
| BB-PRF-004 | FIXED | Bench real file shapes; bound per-frame cost by what fits on screen | `SYS/PRF/S01/BB_PRF_004_EVERY_FRAME_SHAPED_AND_STYLED_THE_WHOLE_LINE.md` |
| BB-PRF-005 | FIXED | Measure per process by name; beat the run-to-run spread | `SYS/PRF/S02/BB_PRF_005_THE_DOTNET_TOOL_LAUNCHER_STAYED_ALIVE_AS_ROSLYNS_PARENT.md` |
| BB-PRF-006 | FIXED | Keep a plain reference in tests; bench every shape | `SYS/PRF/S02/BB_PRF_006_FIND_IN_FILES_READ_ONE_FILE_AT_A_TIME.md` |
| BB-UIX-015 | FIXED | A write checks the file is still the one read; data loss is never silent | `APP/UIX/D2/S04/BB_UIX_015_SAVE_NEVER_COMPARED_THE_FILE_WITH_WHAT_IT_HAD_READ.md` |
| BB-UIX-016 | FIXED | Anything shown from disk is checked against the disk again | `APP/UIX/D2/S04/BB_UIX_016_THE_EXPLORER_READ_EACH_FOLDER_ONCE.md` |
| BB-PRC-005 | FIXED | An owner's name is as unique as its users; a flake that names its cause is a bug report | `SYS/PRC/S02/BB_PRC_005_TWO_WORKERS_SHARED_ONE_OWNED_TEMP_FOLDER.md` |
| BB-BLD-003 | FIXED | Merged is not installed: name the running build | `SHIP/BLD/BB_BLD_003_A_MERGED_FIX_WAS_REPORTED_BEFORE_IT_WAS_INSTALLED.md` |
| BB-GAT-005 | FIXED | Wait only for what a step already started | `SHIP/GAT/S02/BB_GAT_005_A_TEST_WAITED_FOR_A_SEARCH_IT_HAD_NOT_STARTED.md` |
| BB-GAT-006 | FIXED | Never edit a worktree while its gate runs | `SHIP/GAT/S02/BB_GAT_006_A_WORKTREE_WAS_EDITED_WHILE_ITS_GATE_RAN.md` |
| BB-ENV-002 | FIXED | Heavy jobs on the headmate's Mac run with one worker | `SYS/ENV/BB_ENV_002_A_GATE_ON_EVERY_CORE_OVERHEATED_THE_MAC.md` |
| BB-PRC-008 | FIXED | Join a detached owner before returning; pipe test runs to a file | `SYS/PRC/S02/BB_PRC_008_A_DEBUG_TEST_RUN_OUTLIVED_ITS_TEST_BINARY.md` |
| BB-GAT-008 | FIXED | Clippy on touched crates before the gate; audit does not replace it | `SHIP/GAT/S02/BB_GAT_008_A_TEST_WILDCARD_ARM_REACHED_THE_GATE_UNLINTED.md` |
| BB-GAT-009 | FIXED | Run a variant-dependent live test on the stock adapter too | `SHIP/GAT/S03/BB_GAT_009_A_PATCHED_LOCAL_ADAPTER_HID_A_CI_ONLY_BRANCH.md` |
| BB-GAT-010 | FIXED | Arm the watchdog on every wait that can block; run new live tests under an alarm | `SHIP/GAT/S03/BB_GAT_010_A_WINDOW_TEST_WAITED_WITH_NO_DEADLINE_THAT_COULD_FIRE.md` |
| BB-GAT-011 | FIXED | Test line-break counts for every literal form | `SHIP/GAT/S03/BB_GAT_011_THE_LEXER_DROPPED_AN_ESCAPED_LINE_BREAK.md` |
| BB-GAT-012 | OBSERVED | Count parallel-tool output with more than one job | `SHIP/GAT/S03/BB_GAT_012_ONE_LOCAL_BUILD_JOB_HID_A_DUPLICATE_CI_ERROR.md` |
| BB-LSP-009 | FIXED | Run a real-server live test more than once; retry codes are the client's | `APP/LSP/S03/BB_LSP_009_A_CONTENT_MODIFIED_REPLY_WAS_SHOWN_AS_A_FAILURE.md` |
| BB-LSP-010 | FIXED | Route every user text by the server that answered | `APP/LSP/S03/BB_LSP_010_A_FAILED_ANSWER_WAS_NAMED_AFTER_ROSLYN.md` |
| BB-LSP-011 | FIXED | Every answer kind handles its empty case | `APP/LSP/S03/BB_LSP_011_AN_EMPTY_RENAME_PLAN_OPENED_A_PREVIEW.md` |
| BB-ENV-003 | FIXED | A new tool adds its install folder to the fallback; read back with a failing shell | `SYS/ENV/BB_ENV_003_THE_FALLBACK_PATH_MISSED_RUSTUPS_FOLDER.md` |
| BB-GAT-013 | FIXED | A variant green counts for its branch; prove the other with BACKBACK_REQUIRE_TOOLS | `SHIP/GAT/S04/BB_GAT_013_A_VARIANT_TESTS_REAL_TOOL_BRANCH_HAD_NEVER_RUN.md` |
| BB-GAT-014 | FIXED | Real tools reach a test window through the test thread | `SHIP/GAT/S04/BB_GAT_014_A_REAL_SERVER_THREAD_WOKE_GPUIS_TEST_SCHEDULER.md` |
| BB-GAT-015 | FIXED | A deadline test gives the start its own deadline | `SHIP/GAT/S04/BB_GAT_015_A_SHORT_REQUEST_DEADLINE_ALSO_BOUNDED_THE_SERVER_START.md` |
| BB-UIX-017 | FIXED | A control's live state and its action share one predicate | `APP/UIX/D2/S05/BB_UIX_017_BACK_NEEDED_AN_OPEN_TAB_AND_ITS_ARROW_TRUSTED_A_NON_EMPTY_STACK.md` |
| BB-UIX-018 | FIXED | Whatever waits for an async result is cleared on its failure arm too | `APP/UIX/D2/S05/BB_UIX_018_A_FAILED_OPEN_KEPT_THE_JUMP_THAT_WAITED_FOR_IT.md` |
| BB-UIX-019 | FIXED | A store keyed by path is checked against the disk where it is used | `APP/UIX/D2/S05/BB_UIX_019_THE_BREAKPOINT_STORE_OUTLIVED_ITS_FILE.md` |
| BB-UIX-020 | FIXED | One error arm for two causes is split by a known fact | `APP/UIX/D2/S05/BB_UIX_020_A_PATH_THAT_DID_NOT_RESOLVE_WAS_ALWAYS_CALLED_REMOTE.md` |
| BB-UIX-021 | FIXED | Doc'd C#-only rules must grow with each new language | `APP/UIX/D2/S06/BB_UIX_021_A_FILE_ARGUMENT_LOOKED_ONLY_FOR_A_CSHARP_SOLUTION.md` |
| BB-UIX-022 | FIXED | Grep the old language's literal when chrome becomes language-aware | `APP/UIX/D2/S06/BB_UIX_022_CSHARP_ONLY_CHROME_WAS_NOT_TIED_TO_THE_WINDOWS_LANGUAGES.md` |
| BB-UIX-023 | FIXED | Keep a list's heading with the list; key-driven lists scroll | `APP/UIX/D2/S06/BB_UIX_023_THE_START_PAGE_LIST_HAD_NO_SCROLL_HANDLE_AND_ITS_HEADING_SAT_IN_THE_HEADER.md` |
| BB-UIX-024 | FIXED | A panel shows one line with the place; the full text on hover | `APP/UIX/D2/S06/BB_UIX_024_A_TOOLS_WHOLE_STDERR_WAS_SHOWN_IN_A_NARROW_SECTION.md` |
