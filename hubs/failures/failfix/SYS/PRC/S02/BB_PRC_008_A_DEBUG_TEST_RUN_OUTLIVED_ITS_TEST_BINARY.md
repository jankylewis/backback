# BB-PRC-008 — fix

id: BB-PRC-008
family: PRC
status: FIXED
source: branch `fix/five-debug-window`, `session/tests/guard.rs`

## The fix that holds

Every real-session test holds a `Debugged` guard: its drop (on success and
on a panic's unwind) sends Stop and waits until the session thread has
closed its channel, which it does only after it reaped the run.

## The wrong fix

Killing the run in the test by pid: the thread still dies mid-teardown,
and the next detached owner leaks again. Sibling still open: the intel
worker thread in tests (BB-PRC-007).

## Proof

`a_dropped_debug_test_session_leaves_no_test_run`: red with an empty
drop ("the run outlived its session"), green with the guard; after the
five session tests, `pgrep` finds no `dotnet test` on a temp project.
