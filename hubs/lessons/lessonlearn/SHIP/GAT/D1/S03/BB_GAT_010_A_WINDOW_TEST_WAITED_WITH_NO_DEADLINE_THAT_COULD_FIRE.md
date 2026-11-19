# BB-GAT-010 — lesson

id: BB-GAT-010
family: GAT
status: FIXED
source: matemaster stop of the BBEOD27 run, 2026-11-19

## Who believed what

A wait loop with a deadline in its `while` cannot hang.

## What changed the belief

The loop's body (`run_until_parked`) ran a real process to its own end,
so the deadline check never came round; the run idled for minutes.

## Rule left behind

Every wait in a test that can block arms the watchdog with what it waits
for; run new live tests one by one under an alarm, output to a file.
