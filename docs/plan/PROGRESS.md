# Backback progress

- BBEOD43 (`bbeod-43-no-compiler-server`, 2026-11-21 23:18-2026-11-22 00:05, READY: RED 2 compiles served by a shared server, GREEN by design; 6/6 mutants killed; gate 783/0): every `dotnet` from `bb_dotnet::no_servers` (own group, no compiler server or node; guard test) (BB-PRC-010); TRIAGE shelved by layer/cabinet (PRC-009, GAT-019/020 rows).
- BBEOD42 (`bbeod-42-no-orphan-servers`, 2026-11-20 21:05 (est.)-22:37, READY: 5/8 red at 5ad7d19, 9/9 green, 14/15 mutants killed): no child outlives backback (BB-PRC-009, BB-BUG-094): a `/bin/sh` guard on a pipe ends every owned group however backback ends; launch sweeps dead owners' `children/` records; debugger build grouped; worker and live window tests drain their servers (BB-PRC-007 LEAK). CI fix-forward (run 44768652825): self-ending scripts hold at `wait:go` until the test saw its processes (BB-GAT-020).
- BBEOD41 (`bbeod-41-python-opt-in`, 2026-11-20 14:34-15:11, READY: gate 440/0, 13/13 mutants killed): Python opt-in ruling; default ticks drop Python (Python-only folder keeps it), choice saved only when picked (store v2 `picked` mark, v1 = no choice), Python ignored window-wide when unticked.
- BBEOD39 (`test/late-langs-real`): late languages on real servers, mid-parse recolour; BB-LSP-013 (restart reopened every tab); CI red fixed: walk hold until released, servers end before a test returns (BB-GAT-018).
- BBEOD38 (`perf/first-frame-no-wait`): the window no longer waits for the language walk; languages, menus, servers, colours follow it (BB-PRF-009 FIXED; BB-PRF-008 ACCEPTED).
- BBEOD37 (`chore/gat-shelves-and-size`): GAT in drawers, BB-GAT-017; size by crate (BB-PRF-008), dna-ax first open (BB-PRF-009).
- BBEOD34 (`ci/real-tools`): CI runs real rust-analyzer, pyright, pytest (BB-GAT-016). BBEOD29 (`fix/real-tools`, 2026-11-19): real tools proven; BB-BUG-076..079, BB-LSP-012.
- BBEOD27/28 (#116/#117): rust-analyzer, `bb-cargo` (BB-GAT-010..012); pyright, `bb-python`.
- BBEOD26 (#115, `feat/lang-picker`): C#/Rust/Python ticks; active = ticked
  and present gates every tool, colour and Debug. ADR-11. BBEOD25: #114.

- FIND-SCOPE (2026-11-07, `find-in-files-scope`): Find in Files gets
  Wu / VS Code / Rider fields. Tab moves text -> in -> files. "in" picks
  the workspace, the file's C# project or folder, or any project (typed
  text narrows); "files" takes globs (`*.cs, !*.md`, `!Tests`), with
  .gitignore still applied. Explorer right-click: Find in Folder....

- CI-WORKERS (2026-11-06, `ci-nextest-workers`): CI runs `cargo nextest`
  workers sized to the runner's RAM; deny and nextest prebuilt and pinned.

- CLOSE-FOLDER (2026-11-06, `close-folder-recent10`): File > Close Folder
  (Cmd+K F) closes the workspace (unsaved tabs ask first) and shows the
  start page. The recent list keeps 10 and drops deleted folders (test
  temp folders took 5 of 8 places). Window tests for both close paths.

- CODE-MAP (2026-11-05, `code-map`): `bb map` (crate `bb-kmap`) for agents:
  crate index, file outline with line numbers, `--find`. Read from source
  each call (never stale). BB-TG-020. README, architecture, conventions and
  roadmap brought up to date (16 crates, gate.sh, shipped features).

- BBEOD08-REAP (`backback_2`): `bbeod-08-reap` from `1391b5c`.
  Shared group ownership covers testkit, Roslyn, and DAP. Red-first tests passed. Gate: 304 passed in 49 suites,
  0 failed or ignored; format, Clippy, deny, and audit passed. Desktop SIGTERM
  reaped two Roslyn processes; GUI test and debug remain unverified
  (`readback/bb_FOUNDATION/BBEOD08/REAP.md`); bench was noisy
  (load1 7.08), with no regression verdict
  (`bench/bb_GENERAL/TEST_DISCOVERY/BBEOD08_REAP.md`). No push.

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
  `bench/BBEOD06/BBEOD06_GUI.md`, `bench/BBEOD06/bb_BBEOD06_MCP.md`.

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
`readback/V1/` and `git log`. Bugs and failures: `hubs/failures/TRIAGE.md`.
