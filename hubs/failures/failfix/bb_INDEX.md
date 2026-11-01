# Failfix index

| Id | Status | Fix | Path |
|---|---|---|---|
| BB-BLD-002 | FIXED | GUI binary is the bundle executable; served app verified | `SHIP/BLD/BB_BLD_002_BUNDLE_OPENED_NO_WINDOW.md` |
| BB-LSP-001 | FIXED | Count projects by distinct path; wait on readiness | `APP/LSP/BB_LSP_001_READINESS_WAIT_ON_A_FIXED_WALL.md` |
| BB-PRF-001 | FIXED | Files over 4 MiB parse ±1,000 lines around the view | `SYS/PRF/BB_PRF_001_WHOLE_FILE_SYNTAX_TREE_HOLDS_THE_HEAP.md` |
| BB-UIX-001 | FIXED | One shaped line per row: caret quad, clicks, popups and hover share `Columns` | `APP/UIX/BB_UIX_001_CURSOR_DRAWN_AS_A_TEXT_CHARACTER.md` |
| BB-LSP-002 | FIXED | `first_diagnostics` skips late Progress, names any other event | `APP/LSP/BB_LSP_002_TEST_TAKES_THE_FIRST_EVENT_AFTER_OPEN.md` |
| BB-CIP-001 | FIXED | CI installs pinned dotnet, Roslyn server and netcoredbg; `--no-fail-fast` | `SHIP/CIP/BB_CIP_001_CI_RUNNER_HAS_NO_ROSLYN_SERVER.md` |
| BB-CIP-002 | FIXED | `runs-on: macos-latest` | `SHIP/CIP/BB_CIP_002_RUNNER_LABEL_NEVER_GETS_A_MACHINE.md` |
| BB-UIX-002 | FIXED | Minimize/Zoom handled on the root view with its own Window | `APP/UIX/BB_UIX_002_MINIMIZE_ON_THE_ACTIVE_WINDOW.md` |
| BB-PRC-001 | FIXED | Own every child's process group; every exit ends the registry | `SYS/PRC/BB_PRC_001_A_CHILD_GROUP_OUTLIVES_ITS_OWNER.md` |
| BB-UIX-003 | FIXED | Gutter width scales with the size | `APP/UIX/BB_UIX_003_GUTTER_WIDTH_FIXED_WHEN_ZOOMED.md` |
| BB-PRC-002 | FIXED | Spawn on the UI thread, wait in the background | `SYS/PRC/BB_PRC_002_BUILD_SPAWNED_FROM_A_BACKGROUND_THREAD_HANGS.md` |
| BB-BLD-001 | FIXED | `install.sh` from clean `master` to `/Applications`; `BBCommit` stamp | `SHIP/BLD/BB_BLD_001_INSTALL_FROM_MASTER.md` |
