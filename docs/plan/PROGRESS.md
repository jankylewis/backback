# BBEOD05 ship progress

## BBEOD05-EXITCODE (backback_1)

- Local netcoredbg `9731de2` on `local-setnext`: kqueue `NOTE_EXITSTATUS`
  gives the launch exit code; capability `netcoredbgLaunchExitCodeIsProcessStatus`;
  a signal death adds `netcoredbgSignal`.
- Backback: `launch_exit_truth` is `Verified` (or `Signaled`) only with that
  capability; stock stays `Unverified`. Readback: `readback/BBEOD05/EXITCODE.md`.
- Gate: stock and patched 159 passed, 0 failed, 0 ignored (30 suites); no
  adapter: 13 adapter tests fail with `NotFound`. No bench (no hot path).

## BBEOD05-FIX (backback_1)

- Branch `bbeod-05-ship` from `1d1b791`; tree `backback_wt/bbeod-05-ship`.
- B1 liveness from adapter exit; B2 attach target exit; B3 exit code truth;
  B4 no vacuous skip (audit rule `VacuousSkip`); typed condition error.
- Gate: stock and patched adapters 153 passed, 0 failed, 0 ignored (30
  suites); no adapter: 12 adapter tests fail with a clear message.
- Readback: `readback/BBEOD05/FIX.md`; bench: `bench/BBEOD05/FIX.md` (noisy).
- Upstream draft: `docs/rnd/NETCOREDBG_EXIT_CODE.md`.

- Seat: `backback_2`; branch: `bbeod-05-ship`; base: `19050c9`.
- Merged `bbeod-05-debug` and `bbeod-05-setnext` with both intents kept.
- Stock and local patched netcoredbg each passed the capability-aware Set Next instrument.
- Merged gate with the local patched adapter passed: fmt, Clippy, 131 tests,
  deny, bb audit, and tracked absolute-path guard. Independent review is pending.

## Earlier work

- BBEOD05-B Set Next (`backback_2`, 2026-11-01): local netcoredbg
  `3.2.0-1092` clone at `backback_wt/netcoredbg`, branch `local-setnext`.
  Backback instrument branch `bbeod-05-setnext` starts at origin/master.
  Local DAP goto patch built and installed. Upstream-style fixture passed for
  same-method re-execution, stale IDs, cross-method, catch, and finally gates.
  Backback live DAP instrument passed. Merged backback gate: fmt, Clippy, 127 tests,
  deny, and `bb audit` passed. Full netcoredbg build and 6 CTest tests passed.
  Direct upstream-style runner passed; no publication.

- Hygiene-ctx (`backback_2`, 2026-11-01): moved five tracked ctx summaries
  unchanged to the external rawctxhub; removed the repo copies and added an
  ignore rule and external-hub instruction. Public-path review found two
  absolute home paths; a tracked-text guard failed on both before removal.
  Revised gate: fmt, Clippy, 126 tests, deny, and `bb audit` passed.
## BBEOD05-A debugger engine in progress

- Seat: `backback_2`; branch: `bbeod-05-debug`; base: `978f75f`.
- Official netcoredbg `3.2.0-1092` macOS arm64 archive passed its published SHA-256 check.
- Next: typed DAP session, real debugger probe, gate, readback, and benchmark evidence.
- DAP attach readback: GreeterSpec stopped, locals were visible, step resumed, and both tests finished.
- Probe CLI stopped at a temporary console fixture and printed stack and locals.
- Measured attach-to-first-stop 594 ms and step-to-stop 19 ms in one real run.
- Final gate: format, Clippy, 87 tests, and cargo-deny all passed.
- Performance: five-run release benchmark completed at load1 5.02; quiet comparison remains open.
- Remaining: handoff summary, commit, and READY to `backback_mm`.

## BBEOD05-A acceptance benchmark

- Functionally accepted; performance comparison is in progress.
- A: master `978f75f`; B: `bbeod-05-debug` `e01277d`.
- Plan: release builds, five interleaved A/B pairs, three measured runs per item.
- Target items: scan-harness, scan-solution, and audit-repo.
- The redundant test-only Clippy allow was removed; `#![cfg(test)]` kept the integration helpers in test scope.
- A/B second window: median paired B deltas +2.7% harness, −1.5% solution, +3.8% audit.
- A/A control and sign-changing pairs support `noise, no regression`.

## Prior work

- BBEOD01 plan, conventions, and research were handed off; the audit was clean.
- BBEOD02 editor, Explorer, theme, and fixes B1-B3 were tested on fixture,
  MbTx, and Apex readback. Evidence remains in `bench/` and `readback/BBEOD02/`.
- BBEOD02 editor p99 was 22.036 ms for 500 paced keys. A 47.95 Hz display
  imposed a 20.833 ms frame floor; the quiet-host check remains open.
- BBEOD02 Explorer MbTx expand-all median grew from 576 ms at 8,758 folders
  to 664 ms at 8,712 folders. The indicative +15.3% cost remains recorded.
- BBEOD02 review fixed the depth-7 guide and verified resizable columns,
  truncation, and icons. Evidence remains in `readback/BBEOD02/`.
