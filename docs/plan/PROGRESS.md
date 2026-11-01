# Backback progress

- BBEOD06-FIX2 (backback_1, 2026-11-01): `bbeod-06-gui` merged with master
  `0628b87`; B1 orphan group on signal/drop/quit, B2 Stop cancels a hung
  Start. Lessons: `readback/BBEOD06/FIX/bb_FIX.md`.
- BBEOD06-FIX (backback_1): B3-B8 + F1 fixed test-first on `9d7ea08`.
- BBEOD06-B (backback_2): MCP harness panel + engine; bench
  `bench/MCP/BBEOD06_GUI.md`, `bench/MCP/bb_BBEOD06_MCP.md`.

- BBEOD04-B merge (backback_2, 2026-11-01): `origin/master` plus
  cancellation, in-place `TargetPath`, temp stub. GPUI passed; gate: fmt,
  Clippy, 234 tests, deny, audit clean. One earlier Roslyn batch was red.
- BBEOD04-B (backback_2, 2026-11-01): `bbeod-04-gui` from `a10a5f3`.
  Test Explorer GUI readback: two specs, one pass/fail, failure jump.
  Prior gate: 230 tests; bench 996.2 ms at load1 10.63, quiet check open.
  Debug awaits BBEOD05. Records: `readback/BBEOD03/BBEOD04_B/`,
  `bench/TEST_DISCOVERY/bb_BBEOD04_B.md`.
- BBEOD03-FIX2 (backback_1, 2026-11-01): TargetPath stub, MbTx 11/11,
  0 writes, diagnostics clear. `readback/BBEOD03/BBEOD03_FIX/BBEOD03_FIX2.md`.
- BBEOD03-FIX (backback_2, 2026-11-01): merge `c07f7bf`, temp discovery;
  child death gives Stopped within 1 s; restart works. Gate: 224 tests,
  fmt, Clippy, deny, audit. `readback/BBEOD03/BBEOD03_FIX/bb_BBEOD03_FIX.md`,
  `bench/TEST_DISCOVERY/BBEOD03_FIX.md`. Quiet timing remains open.

- BBEOD02-MEM (backback_1, 2026-11-01): 20 MB file RSS 1,008 -> 166 MB; the
  holder was the whole-file tree-sitter tree (862 MB). Large files now parse
  a window around the view. Record: `bench/BBEOD02/BBEOD02_FIX.md`.

- Hygiene-ctx (`backback_2`, 2026-11-01): moved five ctx summaries to
  external rawctxhub; added ignore and external-hub rule. Removed two
  absolute home paths found by a failing tracked-text guard. Revised gate:
  fmt, Clippy, 126 tests, deny, and `bb audit` passed.

- BBEOD02-FIX (backback_1, 2026-11-01): fixes for verification bugs B1-B3.
  B1 safe save, B2 rope edit log + background open/parse (key p99 244 ms
  -> 2.6 ms), B3 virtual Explorer (frame 439 -> 197 ms, RSS 471 -> 112 MiB),
  status bar, natural sort, Quit. Evidence: `bench/BBEOD02/BBEOD02_FIX.md`,
  `readback/BBEOD02/FIX/`. Open: FS watcher, 1 GB RSS on 20 MB file.

- BBEOD02-B (backback_2): rope editor, guarded tabs, atomic save, C#
  highlighting, light theme + View toggle; 500-key p99 22.036 ms at a 47.95 Hz
  display floor; quiet-host check open. Evidence: `bench/BBEOD02/EDITOR.md`,
  `readback/BBEOD02/EDITOR/bb_EDITOR.md`. Last gate 109 tests.

- BBEOD02-A Explorer (backback_2): GPUI shell, model tests, screenshots in
  `docs/plan/readback/BBEOD02/`; first gate 62 tests.

## BBEOD03-A (backback_1, branch `bbeod-03-intel`, worktree `backback_wt/bbeod-03-intel`)

- New crate `edge/intel` (`bb-intel`): `Session` (sync + 7 features), `bb probe`.
- Issues fixed, failing first: load signal before the load (wait for every
  project); completion `null` as the first request; references repeated the
  declaration; an edit inside a CRLF pair drifted the server text; a
  timed-out request left its waiter behind.
- Readback: 16 probes on MbTx and Apex, record `readback/BBEOD03/bb_BBEOD03_A.md`.
- Open: references ~500 ms server floor; session drops log lines after load.

## BBEOD04-A (backback_1, same branch and worktree)

- Ruling (c) done: the session keeps writing server lines after the load
  and waits for the last ones on shutdown; `bb open` drains after shutdown.
- Ruling (b) done: `Session::format`, typed `IntelError::Formatter`;
  issue draft `docs/rnd/ROSLYN_FORMATTER_ISSUE.md`.
- Test engine `edge/dotnet/testkit` (`bb-testkit`): Apex CLI + VSTest + MTP
  backends, TRX/list/settings readers, `bb test list|run`; e2e green.
- MbTx list readback done on a filtered scratch copy (MbTx: 0 writes).
- Gate 140 passed / 0 failed, deny ok. Record `readback/BBEOD03/BBEOD04_A.md`.

## BBEOD03-B (backback_1, branch `bbeod-03-gui`, worktree `backback_wt/bbeod-03-gui`)

- Merge `9cc54ed` (03a5511 + `bbeod-03-intel`): gate 184 passed / 0 failed.
- `370ac90`: GUI code intelligence (`desktop/src/intel/`: server thread,
  model, glue, view); bb_text char log, `replace_many`, write lock.
- Live readback (harness, MbTx, Apex `--no-write`, 0 writes) found 4 issues,
  each fixed failing-first. Record `readback/BBEOD03/BBEOD03_B/`.
