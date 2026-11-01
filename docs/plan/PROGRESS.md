# Backback progress

- BBEOD06-REMOVE (`backback_1`, 2026-11-01): headmate ruling, no MCP.
  Removed `bb-mcp` crate, `bb mcp` commands, desktop Harness panel,
  `_Harness/McpServer` fixture, `Capability::Mcp`. `bb_group` lives on
  in BBEOD08-REAP `bb-process`. AGENTS.md gains the Scope law.

- BBEOD07-MERGE (`backback_2`): merged `origin/master` `6b5e68b` on
  `bbeod-07-release`. Final gate: 301 passed in 46 suites, 0 failed,
  0 ignored; format, all-feature Clippy, deny, audit, path guard passed.
  Bundle opened a visible window and quit with 0 leftover processes.
  Icon corrections and conflict table: `readback/V1/BBEOD07/MERGE.md`.

- BBEOD06-MERGE (`backback_2`): merged MCP panel branch `bbeod-06-gui`
  with master debugger work. Stock full gate: 301 passed, 0 failed,
  0 ignored in 46 suites. MCP panel and Test Explorer GPUI tests passed;
  stock debug fixture stopped at line 14. `readback/V1/BBEOD06/MERGE.md`.
- BBEOD06-FIX2 (`backback_1`, 2026-11-01): B1 orphan group on
  signal/drop/quit, B2 Stop cancels a hung Start. Lessons:
  `readback/V1/BBEOD06/FIX/bb_FIX.md`.
- BBEOD06-FIX (`backback_1`): B3-B8 + F1 fixed test-first on `9d7ea08`.
- BBEOD06-B (`backback_2`): MCP harness panel + engine. Bench:
  `bench/MCP/BBEOD06_GUI.md`, `bench/MCP/bb_BBEOD06_MCP.md`.

## BBEOD05 merge

- Branch `bbeod-05-ship` merged master debugger and Set Next work with
  Test Explorer, code intelligence, and test CLI.
- Stock and patched gates: 267 passed, 0 failed, 0 ignored each;
  41 suites each. Format, all-feature Clippy, deny, audit, path guard passed.
- Served readbacks: patched Set Next, stock `bb debug`, Test Explorer GPUI.
  Evidence and conflict table: `readback/V1/BBEOD05/MERGE.md`.
- Source research found no `runInTerminal` reverse request. No push.

## BBEOD03-LOADCOUNT (backback_1)

- Same branch. Families: load state from summed batch lines; fixed test
  walls around Roslyn readiness. `LoadState` counts distinct paths; tests
  wait on the readiness line, then a request budget; failed projects reach
  `bb open`, `bb probe` and the status bar.
- Gate: 41 suites, 289 passed, 0 failed. Loops 20/20. MbTx 17/17, 0 written.
  Record: `readback/V1/BBEOD03/BBEOD03_FIX/BBEOD03_LOADCOUNT.md`.

## BBEOD03-DIAGVER (backback_1)

- Branch `bbeod-03-diagver` from `3d83ae0`. Family: diagnostics not tied
  to the document version. Versioned store, pull returns its own answer,
  poison is an error, `Event::Diagnostics` carries `DocVersion`, the window
  shows a list only for the version it shows.
- Readback found a first-fix bug: Roslyn "unchanged" after an edit put the
  old list one edit behind. Fix: always pull the full list.
- Gate: 41 suites, 279 passed, 0 failed. Worker loop 20/20 at 2 threads.
- Open: Roslyn reload lines double-count a project (`_Core` twice, sum 5
  of 4). Next unit after BBEOD07-VERIFY2.
- Record: `readback/V1/BBEOD03/BBEOD03_FIX/BBEOD03_DIAGVER.md`.

## Earlier units

BBEOD05 (debugger, Set Next, exit code), BBEOD05-FIX and BBEOD07-FIX: see
`HISTORY.md`, the goals board `bb_roadmap/GOALS.md`, the readbacks under
`readback/V1/` and `git log`. Bugs and failures: `hubs/TRIAGE.md`.
