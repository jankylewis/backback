# BBEOD05 merge progress

- Seat `backback_2`, branch `bbeod-05-ship`, base `bb1d051`.
- Merged `origin/master` (`0628b87`): debugger and Set Next coexist with
  Test Explorer, code intelligence, test CLI, and master document names.
- Stock and patched full gates: 267 passed, 0 failed, 0 ignored each;
  41 suites each. Format, all-feature Clippy, deny, audit, and path guard pass.
- Served readbacks: patched Set Next, stock `bb debug`, Test Explorer GPUI.
  Evidence and conflict table: `readback/BBEOD05/MERGE.md`.
- Source research found no `runInTerminal` reverse request. No push.

## BBEOD05-EXITCODE (backback_1)

- Local netcoredbg `9731de2` on `local-setnext`: kqueue `NOTE_EXITSTATUS`
  gives launch exit code; capability `netcoredbgLaunchExitCodeIsProcessStatus`.
- Backback marks launch status `Verified` or `Signaled` only with that
  capability. Stock is `Unverified`.
- Before merge: stock and patched each passed 159 tests; no adapter caused
  13 adapter tests to fail with `NotFound`.
- Readback: `readback/BBEOD05/EXITCODE.md`.

## BBEOD05-FIX (backback_1)

- B1 liveness from adapter exit; B2 attach target exit; B3 status truth;
  B4 no vacuous skip; typed condition error.
- Before merge: stock and patched each passed 153 tests. No adapter caused
  12 adapter tests to fail with a clear message.
- Records: `readback/BBEOD05/FIX.md`, `bench/BBEOD05/FIX.md`,
  `docs/rnd/NETCOREDBG_EXIT_CODE.md`.

## BBEOD05 debugger and Set Next (backback_2)

- Merged `bbeod-05-debug` and `bbeod-05-setnext` before this master merge.
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

- BBEOD02 through BBEOD04 state from master: `docs/plan/HISTORY.md`.
- Hygiene-ctx moved five summaries to the external rawctxhub and removed
  two tracked absolute home paths. The guard failed before that fix.
