# BB-BUG-064 — fix

id: BB-BUG-064
family: TST
status: FIXED
source: `edge/dotnet/testkit/src/backend/debug.rs`, `tests_panel/actions/bb_backend.rs` (`debug_host`), `run/bb_debug/session/bb_worker.rs` (`open`), `session/control.rs` (`debug_test`), `tests_panel/view/rows/spec.rs` and `rows/menu.rs`

## The change

Debug on a test (the row's Debug, or its right-click menu) builds the
project and starts the test paused for a debugger, with the backend that
runs it: the Apex CLI's `--debug`, or `dotnet test` with
`VSTEST_HOST_DEBUG=1` (new `DotnetBackend::debug`; `DebugHost` moved to
`backend/debug.rs` for both). The session attaches netcoredbg to the test
host's pid, sends every breakpoint and exception filter, and the test runs
on; Stop ends the host too. `--no-write` refuses it, as F5.

## Lesson

A placeholder in the UI is a bug report waiting to happen: when the thing
it waits for ships, wire it in the same EOD or remove it.

## Proof

`debugging_a_test_stops_on_a_breakpoint_inside_it` (real netcoredbg and
dotnet test on an NUnit project): stops in `Hits` on the breakpoint line
with `n = 41`. `a_right_click_on_a_test_offers_run_and_debug` (window test).
Gate green.
