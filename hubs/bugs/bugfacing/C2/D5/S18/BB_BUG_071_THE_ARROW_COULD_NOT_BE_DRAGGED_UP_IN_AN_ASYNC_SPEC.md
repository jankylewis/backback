# BB-BUG-071 — The arrow could not be dragged up in an async spec

id: BB-BUG-071
family: DAP
status: OBSERVED
date: 2026-11-18
signature: drag the ➜ up inside an `async Task` spec: the arrow stays; the status may say `Set Next Statement: line N is not a target`
source: headmate (fourth request, 2026-11-18)
cause: netcoredbg offers a Set Next target only on a line where a statement starts (`GetGotoTargets` matches `startLine` exactly); a later line of a multi-line statement (an `await` chain over two lines, as MbTx specs write) and a lambda's body lines have none, and the refusal was one status line. The patched adapter itself moves the arrow up inside an async `MoveNext`, under launch and under the test host, across awaits (live probes 2026-11-18). Which line the headmate dropped on: not proven
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Debug an async spec, pause after an await, drag the ➜ up onto the second line of a multi-line `await` statement.

Expected: That statement runs next.
Actual: Nothing moves.
