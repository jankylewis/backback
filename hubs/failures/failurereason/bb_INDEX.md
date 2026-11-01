# Failurereason index

One file per failure mechanism. This hub owns the id. Entries: 15 — FIXED 13, OBSERVED 1, OPEN 1.

| Id | Failure reason | Status | fix | lesson | toolgap | Path |
|---|---|---|---|---|---|---|
| BB-BLD-001 | An app built before a fix keeps running from a worktree bundle | FIXED | yes | yes | none | `SHIP/BLD/BB_BLD_001_STALE_APP_RUN_FROM_A_WORKTREE.md` |
| BB-BLD-002 | The `.app` opened no window; a first-frame readback said it did | FIXED | yes | yes | none | `SHIP/BLD/BB_BLD_002_BUNDLE_OPENED_NO_WINDOW.md` |
| BB-CIP-001 | The CI runner has no roslyn-language-server or netcoredbg | FIXED | yes | yes | BB-TG-004 | `SHIP/CIP/BB_CIP_001_CI_RUNNER_HAS_NO_ROSLYN_SERVER.md` |
| BB-LSP-001 | Roslyn tests wait on a fixed wall and count reloads twice | FIXED | yes | yes | none | `APP/LSP/BB_LSP_001_READINESS_WAIT_ON_A_FIXED_WALL.md` |
| BB-PRF-001 | A whole-file syntax tree held 97% of the heap on a 20 MB file | FIXED | yes | yes | none | `SYS/PRF/BB_PRF_001_WHOLE_FILE_SYNTAX_TREE_HOLDS_THE_HEAP.md` |
| BB-UIX-001 | The cursor is drawn as a text character inside the line | FIXED | yes | yes | BB-TG-003 | `APP/UIX/BB_UIX_001_CURSOR_DRAWN_AS_A_TEXT_CHARACTER.md` |
| BB-LSP-002 | A test took the first event after Open as the diagnostics | FIXED | yes | no | none | `APP/LSP/BB_LSP_002_TEST_TAKES_THE_FIRST_EVENT_AFTER_OPEN.md` |
| BB-CIP-002 | A runner label no hosted machine serves keeps the job queued | FIXED | yes | no | none | `SHIP/CIP/BB_CIP_002_RUNNER_LABEL_NEVER_GETS_A_MACHINE.md` |
| BB-LSP-003 | Problems still counted a deleted error 6 s after the edit | OPEN | no | no | none | `APP/LSP/BB_LSP_003_DIAGNOSTICS_LAG_AFTER_A_DELETE.md` |
| BB-UIX-002 | Minimize acted on `cx.active_window()`, None while another app is active | FIXED | yes | yes | BB-TG-005 | `APP/UIX/BB_UIX_002_MINIMIZE_ON_THE_ACTIVE_WINDOW.md` |
| BB-PRC-001 | A child process group outlives its owner | FIXED | yes | yes | none | `SYS/PRC/BB_PRC_001_A_CHILD_GROUP_OUTLIVES_ITS_OWNER.md` |
| BB-LSP-004 | `open --restore` can exit before Roslyn logs the restore | FIXED | yes | no | none | `APP/LSP/BB_LSP_004_RESTORE_LINE_AFTER_OPEN_EXITS.md` |
| BB-UIX-003 | The gutter kept 50 px when zoomed, so line numbers ran into the code | FIXED | yes | no | none | `APP/UIX/BB_UIX_003_GUTTER_WIDTH_FIXED_WHEN_ZOOMED.md` |
| BB-PRC-002 | `dotnet build` spawned from a background thread never finished | FIXED | yes | yes | none | `SYS/PRC/BB_PRC_002_BUILD_SPAWNED_FROM_A_BACKGROUND_THREAD_HANGS.md` |
| BB-PRC-003 | `SN` Roslyn came from a niced readback launch, not the app | OBSERVED | no | no | BB-TG-006 | `SYS/PRC/BB_PRC_003_TOOLS_SPAWNED_FROM_BACKGROUND_THREADS.md` |
