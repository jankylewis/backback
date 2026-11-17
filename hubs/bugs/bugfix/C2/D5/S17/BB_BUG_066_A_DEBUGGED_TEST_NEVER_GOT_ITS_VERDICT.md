# BB-BUG-066 — fix

id: BB-BUG-066
family: TST
status: FIXED
source: `run/bb_debug/session/stops.rs` (`test_results`), `testkit/backend/debug.rs` (`finish`, `--logger trx`)

## The change

When the program ends by itself, the run is given 20 s to write its TRX (`DebugHost::finish`), the session reads it and sends `Note::TestResults`; the window records them as a run's. A Stop leaves no verdict. `dotnet test` under `VSTEST_HOST_DEBUG` now writes a TRX too.

## Lesson

A second path that does the same job (debug a test = run a test) must end in the same place: check every result sink a run feeds.

## Proof

`a_debugged_tests_results_mark_its_row_after_the_session_ends`: red with results applied only while the session lives (mutation).
