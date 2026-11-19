# BB-GAT-009 — A patched local adapter hid a CI-only branch

id: BB-GAT-009
family: GAT
status: FIXED
date: 2026-11-18
signature: `expected a stop, got Refused("Set Next Statement refused: adapter does not support Set Next Statement")`
source: `bb_crates/shell/desktop/src/bb_explorer/panels/run/bb_debug/session/tests/bb_async.rs` (`set_next_moves_up_inside_an_async_spec_and_its_late_local_shows`)
source: PR #114, CI run 44341920278: 606 run, 605 passed, 1 failed
observed: CI runs the stock netcoredbg 3.2.0 (`BACKBACK_NETCOREDBG` set by the workflow); the headmate's Mac and every seat run the local patched build (`supportsGotoTargetsRequest=true`)
links: fix=yes lesson=yes evidence=no toolgap=none

## What it is

The async Set Next test assumed the adapter can move the next statement.
The stock adapter does not advertise `gotoTargets`, so the session sends
the visible `Refused` note instead of a `goto` stop and the test panicked.
Every local gate passed because every local machine has the patched build.
`edge/dap/tests/bb_setnext.rs` was already variant-aware; the new
desktop-layer test was not.

## How to confirm it

Run the test with `BACKBACK_NETCOREDBG` set to a stock netcoredbg copy;
without the variant branch it fails with the signature above.
