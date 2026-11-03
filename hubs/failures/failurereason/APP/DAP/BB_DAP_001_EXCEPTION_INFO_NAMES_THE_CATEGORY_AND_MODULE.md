# BB-DAP-001 — exceptionInfo names the category and the module, not the exception

id: BB-DAP-001
family: DAP
status: FIXED
date: 2026-11-03
signature: CLR/System.InvalidOperationException: Exception thrown:
source: `bb_crates/edge/dap/src/bb_session/inspect.rs`, test `the_all_filter_pauses_on_a_caught_throw_and_names_the_exception` (branch `bb-dap-exception-breaks`)
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

netcoredbg's `exceptionInfo` reply puts the CLR category in front of the
type (`exceptionId` = `CLR/System.InvalidOperationException`) and wraps the
message in a sentence about the module (`description` = `Exception thrown:
'System.InvalidOperationException' in Debuggee.dll: 'total was 14'`). The
bare type and message live one level down, in `details.fullTypeName` and
`details.message`. A headline built from the top-level fields reads
`CLR/System.InvalidOperationException: Exception thrown:
'System.InvalidOperationException' in Debuggee.dll: 'total was 14'`: the
type twice, the module once, the message last.

## How it shows

The live session test asked the headline to start with the type and to hold
the message; it got the string above (red first, on the real netcoredbg).

## How to confirm it

Send `exceptionInfo` for a stopped thread and read the body; the probe in
`docs/plan/readback/V2/BBEOD19/bb_READBACK.md` shows the full reply.
