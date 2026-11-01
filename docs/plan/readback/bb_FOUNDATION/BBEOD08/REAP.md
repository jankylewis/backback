# BBEOD08-REAP readback, 2026-11-01

## Family

**B1: a child group outlives its owner.** Killing only a leader leaves a
runner host, Roslyn descendant, or launched debuggee. A shared `bb-process`
registry owns each group. Drop, app quit, and SIGTERM/SIGHUP end groups with
TERM, 200 ms grace, then KILL. The registry holds a group until cleanup ends.
Shutdown also ends a group registered after shutdown begins. A nonzero group
ID blocks accidental signaling of the caller's process group.

## Failing first and fixed tests

| Owner | Red witness | Green witness |
|---|---|---|
| testkit runner | Cancelled run left a descendant alive | Cancellation reaped it |
| Roslyn child | Dropped client left a descendant alive | Drop reaped it |
| DAP adapter and debuggee | Removing group registration left adapter alive on app quit | App quit reaped both |
| Late registration | Shutdown admitted a later child group | Shutdown ended it |

The DAP red witness used the real installed debugger and the same integration
test with its registration disabled. The test restored registration and passed.
The separate testkit, Roslyn, and DAP tests cover the same family across owners.
The testkit Apex debug host uses the same owned group as ordinary test runs.

## Adjacent sweep

- The desktop quit hook and signal watcher use `bb-process` directly.
- Existing MCP process groups use the shared registry pending the separate
  MCP removal branch. Its build path remains outside this unit by headmate
  ruling; this unit adds no MCP case.
- Other spawns in the inspected edge trees are test fixtures.

## Served readback

- Opened the `_Harness/Harness.slnx` desktop window in an isolated app bundle.
  Its Explorer showed the fixture projects. Recorded the app and its two
  Roslyn processes before SIGTERM. `ps` returned no row for any of the three
  recorded PIDs after SIGTERM. A first bare-binary run gave the same result.
- The served window did not respond to Test Explorer clicks, so a GUI test
  run was not started. The current Test Explorer renders Debug as disabled.
  The live app plus test-run and debug-session combination in the brief is
  therefore **unverified**. The independent process tests above exercised
  real testkit, Roslyn, netcoredbg, and launched debuggee processes.
