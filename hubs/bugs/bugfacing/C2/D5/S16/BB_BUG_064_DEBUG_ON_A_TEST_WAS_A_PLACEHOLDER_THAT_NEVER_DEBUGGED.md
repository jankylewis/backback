# BB-BUG-064 — Debug on a test was a placeholder that never debugged

id: BB-BUG-064
family: TST
status: FIXED
date: 2026-11-11
signature: a test's "Debug" does nothing; breakpoints in the test are never hit
source: headmate report 2026-11-11 ("can't debug this test while set the breakpoints", TESTS panel screenshot)
cause: the TESTS panel drew "Debug" as greyed text with the tooltip "Available once the debugger lands" (`rows/spec.rs`); the debugger shipped (BBEOD17) but was never wired to tests, and the session could only launch a program, never attach to a test host
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Set breakpoints in a test method, then press Debug on that test in TESTS.

Expected: the test runs under the debugger and stops on the breakpoint.
Actual: nothing happens.
