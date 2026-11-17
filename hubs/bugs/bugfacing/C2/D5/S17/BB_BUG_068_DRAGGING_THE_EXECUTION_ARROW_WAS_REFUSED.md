# BB-BUG-068 — Dragging the execution arrow was refused

id: BB-BUG-068
family: DBG
status: FIXED
date: 2026-11-17
signature: drag the ➜ from line 69 to 68: nothing moves; the status says the debugger cannot move the next statement
source: headmate (MbTx, 2026-11-17)
cause: the netcoredbg in `~/.local/share/backback/netcoredbg` was stock 3.2.0, which has no Set Next; the patched build lived only in the build folder. The real Set Next test hid this: on a stock adapter it returns early, and on the patched one it dropped its fixture folder (`fixture::Built`) before launch, so `dotnet` ran the SDK instead of the fixture and timed out
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Debug, pause, drag the ➜ to an earlier line.

Expected: that line runs next.
Actual: refused.
