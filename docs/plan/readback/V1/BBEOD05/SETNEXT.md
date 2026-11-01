# BBEOD05-B Set Next readback

Date: 2026-11-01. Seat: backback_2. The netcoredbg patch is LOCAL and unpublished pending the headmate ruling.

## Family

Name: unsafe instruction-pointer moves from source lines.

Shape: a source line may belong to another method, a stale stop, or an
exception region that CoreCLR cannot exit. A displayed line alone is not a
valid destination.

Guard: enumerate sequence points from the active method only. Offer exact
source-line matches only when `CanSetIP` returns `S_OK`. Tie target IDs to the
stopped thread, module, method, version, and current IL offset. Recheck
`CanSetIP` before `SetIP`; consume IDs after use and clear them on resume.

Known instances: cross-method source line; stale target ID; catch exit;
finally exit. The local upstream-style fixture and backback DAP instrument
cover these cases.

## Served readback

- Baseline tag 3.2.0-1092: DAP initialize omitted
  `supportsGotoTargetsRequest`.
- Local patch: initialize advertises it. A live fixture stopped at line N,
  moved to N-2, stepped twice, and evaluated the count as 4 after re-execution.
- `gotoTargets` returned no cross-method, catch-exit, or finally-exit targets.
- The `goto` response came before `stopped(reason=goto)`; a reused ID failed.

## Limits

- Set Next does not undo side effects. It can run earlier statements again.
- Source lines without precise symbols or `CanSetIP` approval have no target.
- Only the local macOS arm64 build was exercised. No fork or upstream change
  was published.

## Merged ship instrument

- Stock netcoredbg: initialize omitted the goto capability; Set Next stayed
  hidden and the guarded client returned `Unsupported` before sending `goto`.
- Local patched netcoredbg: initialize advertised goto; live N to N-2
  readback, two steps, and count `4` passed. Both runs: 1 passed, 0 ignored.
