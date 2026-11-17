# BB-BUG-068 — fix

id: BB-BUG-068
family: DBG
status: FIXED
source: machine: `~/.local/share/backback/netcoredbg` (patched build, stock kept as `netcoredbg-stock-3.2.0`); `edge/dap/tests/bb_setnext.rs`

## The change

The patched build (local only, never published) is installed; the test keeps `Built` alive for the session.

## Lesson

A test that returns early on the common setup is a vacuous green (family of BBEOD05's VacuousSkip): run it on the setup that takes the full path before trusting it.

## Proof

`setnext_reexecutes_and_refuses_unsafe_targets` on the installed adapter: was Elapsed after 15 s, now passes (1.7 s).
