# Backback progress

- BBEOD07-MERGE (`backback_2`): merged `origin/master` `6b5e68b` on
  `bbeod-07-release`. Final gate: 301 passed in 46 suites, 0 failed,
  0 ignored; format, all-feature Clippy, deny, audit, path guard passed.
  Bundle opened a visible window and quit with 0 leftover processes.
  Icon corrections and conflict table: `readback/BBEOD07/MERGE.md`.

- BBEOD06-MERGE (`backback_2`): merged MCP panel branch `bbeod-06-gui`
  with master debugger work. Stock full gate: 301 passed, 0 failed,
  0 ignored in 46 suites. MCP panel and Test Explorer GPUI tests passed;
  stock debug fixture stopped at line 14. `readback/BBEOD06/MERGE.md`.
- BBEOD06-FIX2 (`backback_1`, 2026-11-01): B1 orphan group on
  signal/drop/quit, B2 Stop cancels a hung Start. Lessons:
  `readback/BBEOD06/FIX/bb_FIX.md`.
- BBEOD06-FIX (`backback_1`): B3-B8 + F1 fixed test-first on `9d7ea08`.
- BBEOD06-B (`backback_2`): MCP harness panel + engine. Bench:
  `bench/MCP/BBEOD06_GUI.md`, `bench/MCP/bb_BBEOD06_MCP.md`.

## BBEOD05 merge

- Branch `bbeod-05-ship` merged master debugger and Set Next work with
  Test Explorer, code intelligence, and test CLI.
- Stock and patched gates: 267 passed, 0 failed, 0 ignored each;
  41 suites each. Format, all-feature Clippy, deny, audit, path guard passed.
- Served readbacks: patched Set Next, stock `bb debug`, Test Explorer GPUI.
  Evidence and conflict table: `readback/BBEOD05/MERGE.md`.
- Source research found no `runInTerminal` reverse request. No push.

## BBEOD05-EXITCODE (`backback_1`)

- Local netcoredbg `9731de2` on `local-setnext`: kqueue `NOTE_EXITSTATUS`
  gives launch exit code; capability `netcoredbgLaunchExitCodeIsProcessStatus`.
- Backback marks launch status `Verified` or `Signaled` only with that
  capability. Stock is `Unverified`.
- Before merge: stock and patched each passed 159 tests; no adapter caused
  13 adapter tests to fail with `NotFound`.
- Readback: `readback/BBEOD05/EXITCODE.md`.

## BBEOD05-FIX (`backback_1`)

- B1 liveness from adapter exit; B2 attach target exit; B3 status truth;
  B4 no vacuous skip; typed condition error.
- Before merge: stock and patched each passed 153 tests. No adapter caused
  12 adapter tests to fail with a clear message.
- Records: `readback/BBEOD05/FIX.md`, `bench/BBEOD05/FIX.md`,
  `docs/rnd/NETCOREDBG_EXIT_CODE.md`.

## BBEOD05 debugger and Set Next (`backback_2`)

- Merged `bbeod-05-debug` and `bbeod-05-setnext` before the master merge.
- Stock and patched netcoredbg passed the capability-aware Set Next instrument.
- Before merge: patched gate passed fmt, Clippy, 131 tests, deny, audit,
  and tracked absolute-path guard.
- Local DAP goto patch passed same-method re-execution, stale IDs,
  cross-method, catch, and finally gates. Full netcoredbg build and six CTests
  passed. No publication.
- Debugger readback stopped GreeterSpec, showed locals, stepped, and finished
  both tests. Probe CLI stopped at a console fixture and showed stack and locals.
- Five-pair A/B benchmark was noisy, with no evidenced regression.
- Records: `readback/BBEOD05/`, `bench/BBEOD05/`.

## Earlier work

- BBEOD02-FIX (`backback_1`, 2026-11-01): B1 safe save; B2 rope edit log and
  background open/parse; B3 virtual Explorer, status bar, natural sort, Quit.
  Evidence: `bench/BBEOD02/BBEOD02_FIX.md`, `readback/bb_FOUNDATION/BBEOD02/FIX/`.
  Open: FS watcher, 1 GB RSS on 20 MB file.
- BBEOD02-B (`backback_2`): rope editor, guarded tabs, atomic save, C#
  highlighting, light theme and View toggle. Evidence:
  `bench/bb_GENERAL/EDITOR.md`,
  `readback/bb_FOUNDATION/BBEOD02/bb_EXPLORER/EDITOR/bb_EDITOR.md`.
- BBEOD02 through BBEOD04 details: `docs/plan/HISTORY.md`.
- Hygiene-ctx moved five summaries to the external rawctxhub and removed
  two tracked absolute home paths. The guard failed before that fix.

## BBEOD07-FIX (backback_2)

- Branch `bbeod-07-release`; merged `origin/master` at `3d83ae0`.
- Prior BBEOD07 release bundled the CLI. This fix bundles the desktop GUI.
- Served `.app` check passed: GUI executable, visible window screenshot,
  app quit, and zero leftover processes for this bundle.
- The first icon had a pointed fringe and a two-chevron pack mark.
  BBEOD07-MERGE corrected both; the archive has 10 sizes.
- CI uses pinned action commits and the all-feature gate. The two HD folders
  now have five entries each and one marker.
- Initial gate found two new marker issues. Both were corrected. The final
  gate passed format, all-feature Clippy, 267 tests in 41 suites, deny,
  audit, and the tracked absolute-path guard.
- Five-run benchmark at load 9.41 is noisy. No performance verdict.
- Records: `readback/BBEOD07/bb_BBEOD07_FIX.md`,
  `bench/bb_GENERAL/BBEOD07_FIX.md`. No push.
