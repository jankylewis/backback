# Failurereason index

One file per failure mechanism. This hub owns the id. Entries: 6 — FIXED 2, OBSERVED 4.

| Id | Failure reason | Status | fix | lesson | toolgap | Path |
|---|---|---|---|---|---|---|
| BB-BLD-001 | An app built before a fix keeps running from a worktree bundle | OBSERVED | no | yes | none | `SHIP/BLD/BB_BLD_001_STALE_APP_RUN_FROM_A_WORKTREE.md` |
| BB-BLD-002 | The `.app` opened no window; a first-frame readback said it did | FIXED | yes | yes | none | `SHIP/BLD/BB_BLD_002_BUNDLE_OPENED_NO_WINDOW.md` |
| BB-CIP-001 | The CI runner has no roslyn-language-server or netcoredbg | OBSERVED | no | no | none | `SHIP/CIP/BB_CIP_001_CI_RUNNER_HAS_NO_ROSLYN_SERVER.md` |
| BB-LSP-001 | Roslyn tests wait on a fixed wall and count reloads twice | OBSERVED | yes | yes | none | `APP/LSP/BB_LSP_001_READINESS_WAIT_ON_A_FIXED_WALL.md` |
| BB-PRF-001 | A whole-file syntax tree held 97% of the heap on a 20 MB file | FIXED | yes | yes | none | `SYS/PRF/BB_PRF_001_WHOLE_FILE_SYNTAX_TREE_HOLDS_THE_HEAP.md` |
| BB-UIX-001 | The cursor is drawn as a text character inside the line | OBSERVED | no | no | none | `APP/UIX/BB_UIX_001_CURSOR_DRAWN_AS_A_TEXT_CHARACTER.md` |
