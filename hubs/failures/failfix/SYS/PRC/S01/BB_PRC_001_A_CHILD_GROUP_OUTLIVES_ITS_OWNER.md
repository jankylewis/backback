# BB-PRC-001 — fix

id: BB-PRC-001
family: PRC
status: FIXED
source: `bb_crates/bb_core/process` (`bb-process`: `OwnedGroup`, `shutdown`, `exit_on_signal`)

## The fix that holds

Every child starts in its own process group, owned by one registry. Drop,
app quit and SIGTERM/SIGHUP end each group: TERM, 200 ms grace, then KILL.
Shutdown also ends a group registered after it began. Owners: testkit
runner, Roslyn client, DAP adapter and debuggee.

## The wrong fix

Killing the leader pid only; or `kill_on_drop` alone (it reaches one pid).

## Proof

Red-first tests per owner (backback_2); real-app readback: quit, Cmd+Q and
SIGTERM each left no recorded process (`docs/plan/readback/FIXES/S01/BBEOD08/`).
