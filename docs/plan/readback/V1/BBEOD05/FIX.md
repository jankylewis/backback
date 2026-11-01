# BBEOD05-FIX debugger lifecycle readback (2026-11-01)

- Seat: `backback_1`; branch: `bbeod-05-ship` from `1d1b791`.
- Fixture: `bb_crates/edge/dap/tests/fixture` (`GotoFixture`), copied to a
  scratch folder for `bb debug`; modes `spin`, `await`, `exit3` in `Lifecycle.cs`.
- Adapters: stock netcoredbg `3.2.0-1` (`9744e1f`) and the local patched build.

## Issues, failing tests, fixes

| Bug | Failing test (stock, before fix) | Fix |
|---|---|---|
| B1 adapter death hangs waiters, orphans debuggee | `stackTrace` `Timeout` after 15 s; debuggee left at ppid 1 | exit watcher owns the adapter child; aborts reader, fails waiters, kills the adapter's process group (launch only) |
| B1 sibling: session dropped after its runtime | debuggee 31443 outlived the session (mutation run) | `Drop` sends `SIGKILL` to the group, bounded |
| B2 attach target exits before `configurationDone` | `next_event` `Timeout` at 1 s | pid poll (200 ms) + 250 ms grace, then synthetic `terminated` |
| B3 exit code | adapter `0`, process `5` | `exit_truth`: owned status wins (`Corrected`); launch is `Unverified` |
| Note: bad condition | plain `Stopped` | `Event::ConditionError` from "Breakpoint error" + stop pair |
| B4 vacuous green | old tree: `bb audit` finds 2 `VacuousSkip` | both sites now `expect`; audit rule `VacuousSkip` |

The attach case of B1 (`adapter_death_after_attach_...`) already passed
before the fix: an attached process does not hold the adapter's stdout. It
stays as a guard that backback never kills an attached process.

## User readback (`bb debug`)

```
bb debug GotoFixture.csproj --break _Program.cs:14
stopped   : breakpoint
frame     : _Program.cs:14 Program.Main()
local     : count = 2
exited    : code 0 (adapter-reported, unverified; see docs/rnd/NETCOREDBG_EXIT_CODE.md)
```

Before the fix, `debug.rs` printed the raw `exited` body (`exited    : {body}`),
with no source; this session did not re-run the old binary (unverified).

## Lessons (bug families)

1. **Liveness from pipe EOF.** A child's stdout stays open while any
   descendant holds it. Take liveness from the process exit (`wait`), never
   from EOF alone. Guard: one exit watcher owns the child.
2. **Confident default for an unknown value.** The adapter sends `0` when it
   cannot see the status. Show where a value came from (`ExitTruth`), and
   never print an unverified value as a fact.
3. **Vacuous green.** A test that returns when its tool is missing proves
   nothing. Guard: `bb audit` rule `VacuousSkip` (bare `return` or
   `return Ok(())` under a tool probe, in test code).
