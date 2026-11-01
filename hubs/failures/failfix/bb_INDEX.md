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
