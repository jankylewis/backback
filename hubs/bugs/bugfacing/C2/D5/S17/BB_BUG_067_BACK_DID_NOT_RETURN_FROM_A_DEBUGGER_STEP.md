# BB-BUG-067 — Back did not return from a debugger step into another file

id: BB-BUG-067
family: NAV
status: FIXED
date: 2026-11-17
signature: Step Into opens file B; Cmd+[ does not go back to file A
source: headmate (MbTx, MbStep.ActionAsync, 2026-11-17)
cause: a debugger stop opened its file with `goto_point`, which does not record the place it leaves; and Cmd+[ / Cmd+] were handled only while the editor had focus
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Debug, pause in file A, Step Into a call defined in file B, press Cmd+[.

Expected: back in file A at the call.
Actual: nothing happens.
