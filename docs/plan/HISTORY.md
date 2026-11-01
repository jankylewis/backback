# BBEOD02 progress

- BBEOD04-B merge (backback_2, 2026-11-01): `origin/master` plus
  cancellation, in-place `TargetPath`, temp stub. GPUI passed; gate: fmt,
  Clippy, 234 tests, deny, audit clean. One earlier Roslyn batch was red.
- BBEOD04-B (backback_2, 2026-11-01): `bbeod-04-gui` from `a10a5f3`.
  Test Explorer GUI readback: two specs, one pass/fail, failure jump.
  Prior gate: 230 tests; bench 996.2 ms at load1 10.63, quiet check open.
  Debug awaits BBEOD05. Records: `readback/BBEOD03/BBEOD04_B/`,
  `bench/bb_GENERAL/TEST_DISCOVERY/bb_BBEOD04_B.md`.
- BBEOD03-FIX2 (backback_1, 2026-11-01): TargetPath stub, MbTx 11/11,
  0 writes, diagnostics clear. `readback/BBEOD03/BBEOD03_FIX/BBEOD03_FIX2.md`.
- BBEOD03-FIX (backback_2, 2026-11-01): merge `c07f7bf`, temp discovery;
  child death gives Stopped within 1 s; restart works. Gate: 224 tests,
  fmt, Clippy, deny, audit. `readback/BBEOD03/BBEOD03_FIX/bb_BBEOD03_FIX.md`,
  `bench/bb_GENERAL/TEST_DISCOVERY/BBEOD03_FIX.md`. Quiet timing remains open.

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
  `readback/bb_FOUNDATION/BBEOD02/FIX/`. Open: FS watcher, 1 GB RSS on 20 MB file.

- BBEOD02-B follow-up opened after acceptance: add light theme that follows
  macOS appearance, live View toggle, and screenshots of both modes.
- One rapid-typing performance round remains: compare one change against the
  accepted 500-key release baseline on the same 5,000-line fixture. Keep only
  a faster, green change; record the result in `docs/plan/bench/bb_GENERAL/EDITOR.md`.
- State first: branch `bbeod-02-editor`, last accepted SHA `7c35450`, no push.
- Theme live readback passed: separate View menu, light toggle, return to the
  current macOS dark appearance through Follow, no restart; both PNGs saved.
- One rapid-key scheduling candidate (`window.refresh`) was measured A/B and
  reverted: p99 41.255 vs 41.222 ms with unequal 11.892/13.332 s key loops.
  Two-frame latency remains open; final full gate passed: fmt, Clippy, 109
  workspace tests, and deny.

- BBEOD02-B editor implemented on `bbeod-02-editor`: rope document, guarded
  tabs and window close, atomic save, visible-line C# highlighting, and input
  latency capture from GPUI's input-to-present histogram.
- Full post-editor gate passed: fmt, clippy, 107 tests, deny. Focused Unicode
  incremental-reparse and short-line vertical cursor regressions passed.
- Live fixture, MbTx, and Apex readbacks passed for tab open and rendering;
  temporary fixture save and dirty window close prompt were verified.
- Review trace: 500 paced keys produced 500 presents, p99 22.036 ms on the
  5,000-line fixture. The active display was 47.95 Hz; each GPUI frame
  reported a 20.833 ms refresh interval. This demonstrates a display floor
  above the 16 ms budget on this configuration; quiet-host check remains open.
- Three live editor screenshots were saved and inspected under
  `docs/plan/readback/bb_FOUNDATION/BBEOD02/bb_EXPLORER/EDITOR/`.
- Editor evidence: `docs/plan/bench/bb_GENERAL/EDITOR.md` and
  `docs/plan/readback/bb_FOUNDATION/BBEOD02/bb_EXPLORER/EDITOR/bb_EDITOR.md`.

- BBEOD02-B start: merged `origin/master` into `bbeod-02-editor`; resolved four
  fixture project rename conflicts in favor of the `_*.csproj` backbone names.
- Pre-editor full gate passed: fmt, clippy, 95 tests, deny. Merge `f99746f`.
- Editor risk families: path identity (typed paths and tab dedup); encoding loss
  (BOM/CRLF round-trip test); undo grouping (word-boundary test); stale syntax
  (edit/reparse span test); cursor bounds (checked rope indices); partial save
  (same-directory temporary file plus rename and failure tests).

- BBEOD02-A Explorer (backback_2): GPUI shell, model tests, screenshots in
  `docs/plan/readback/bb_FOUNDATION/BBEOD02/bb_EXPLORER/`; first gate 62 tests.

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
