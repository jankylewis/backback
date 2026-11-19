# BB-GAT-014 — fix

id: BB-GAT-014
family: GAT
status: FIXED
source: `Live::attach` + `Live::pump`

## The fix that holds

The window listens on an in-thread channel (as the fake rigs do); the
test thread moves the real thread's events across with `try_recv` on
every wait step, so no foreign thread wakes the scheduler.

## Proof

Both live window tests 3 of 3 green with `BACKBACK_REQUIRE_TOOLS=1`.

## The wrong fix

Retrying the test, or starting real servers in every test build.
