# BBEOD04-A readback, 2026-11-01 (backback_1)

## Setup

- App: release `bb test list|run`, branch `bbeod-03-intel`. Machine: M4 Pro,
  macOS 27.0.1, SDK 10.0.403, load 4-9 (busy; times are indicative).
- Apex CLI `apex.cli` 0.8.2-pre (the MbTx pin): a copy installed into a
  scratch tool-path from the local NuGet cache; tests run its `apex.dll`.
- Fixture: `fixtures/_Harness`, copied to temp and built there.
- MbTx: list only, never a run. Every discovery writes
  `bin/.../nunit_random_seed.tmp` (NUnit adapter), so listing in place
  would write into MbTx. Listed instead on a scratch copy of project files,
  `.vscode`, `.config`, `bin/`, `obj/`; `.env*`, `*.runsettings`,
  `.AuthState/` and auth-state JSON left out. MbTx check:
  `find MbTx -newer marker` gave 0 entries. Copy removed after.

## MbTx list comparison (11 test projects)

| project | bb apex | bb vstest | `dotnet test --list-tests` | apex list |
|---|---:|---:|---:|---:|
| MbCore | 857 | 857 | 889 (32 repeated case names) | 1432 ms |
| _MbDesktop | 472 | 472 | 472 | 1506 ms |
| MbMobile | 68 | 68 | 68 | 1234 ms |
| MbService | 250 | 250 | 250 | 1278 ms |
| _RgvMbDesktop | 123 | 123 | 123 | 1363 ms |
| MbKm | 22 | 22 | 22 | 1250 ms |
| MbPdfOracle | 141 | 141 | 141 | 1235 ms |
| MbOcrThumbnailOracle | 316 | 316 | 328 (see note) | 1228 ms |
| DbCores.Tests | - | 40 | 40 | 663 ms (vstest) |
| MbAuthCore.Tests | - | 15 | 15 | 626 ms (vstest) |
| MbKeylab.Tests | - | 591 | 595 (see note) | 708 ms (vstest) |

Whole solution: apex 13.58 s, vstest 12.14 s (includes the MSBuild
`TargetPath` call per project). `dotnet test --list-tests`: 0.90-1.72 s
per project. MbCore (verified): a test-case source yields the same
arguments twice (`("Zoom")` x2); the full names are equal, so the FQN list
and the Testkit tree show one leaf. Note: MbOcrThumbnailOracle has 15 and
MbKeylab 7 display names not in the FQN list, and 3 FQNs each not in the
display list; the split into repeats and renames is unverified.

## 1. Issue: a crashed test host listed as 0 specs

- Observed: MbAuthCore.Tests listed `0 specs`, exit 0 (my copy had lost its
  `.deps.json`/`.runtimeconfig.json`). `dotnet vstest` and the Apex CLI both
  exit 0 with an empty list; the only sign is a `TestPlatformException` on
  stderr.
- Family "success judged by exit code alone". Guard: one `Call::discover`
  for every list path (apex, vstest, mtp) fails on an exception line.
- Failing test: `a_list_whose_test_host_crashes_is_an_error_not_zero_specs`.

## 2. Issue: stopped runs left test hosts running

- Observed: after the e2e suite, `testhost.dll` processes of the temp
  copies were still alive. Killing the host makes `dotnet test` start a
  new waiting host; killing the CLI orphans the rest.
- Family "a killed runner leaves its children". Guard: `tree::stop_tree`
  finds the tree with `pgrep -P` while the parent lives and kills it at
  once; used on every timeout and by `DebugHost::stop`.
- Failing tests: `a_timed_out_call_leaves_none_of_its_processes_behind`
  (mutant without the guard: 2 children left),
  `an_apex_debug_run_gives_the_waiting_host_and_stop_leaves_no_process_behind`.
- Open instances, other crates: `bb_lsp` shutdown timeout, `bench` cold
  start timeout (both kill only the program).

## 3. Issue: the harness fixture did not build

- `NU1605`: Apex 0.8.0-pre needs NUnit >= 4.6.1 (fixture: 4.3.2); xunit.v3
  needs `OutputType Exe`. Fixed both; `bb scan` now says `test exe` for
  `_XunitPlaywright`. Added `OutcomeSpec.cs`: one spec per outcome.

## Rulings (BBEOD03-A)

- (c) server lines after the load: failing test
  `server_log_lines_after_the_load_reach_the_log` (0 lines), now kept to
  shutdown; `bb open` drains after shutdown
  (`open_keeps_the_server_lines_until_the_server_has_shut_down`).
- (b) formatter -32000: minimal witness `"[] //"`; typed
  `IntelError::Formatter`; draft `docs/rnd/ROSLYN_FORMATTER_ISSUE.md`.

## Lesson

An exit code is a claim. Check the output that proves the work (a list,
a TRX, a dead process tree), never the code alone.
