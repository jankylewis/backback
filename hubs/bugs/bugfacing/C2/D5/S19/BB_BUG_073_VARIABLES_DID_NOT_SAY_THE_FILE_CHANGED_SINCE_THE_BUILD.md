# BB-BUG-073 — VARIABLES did not say the file changed since the build

id: BB-BUG-073
family: DAP
status: FIXED
date: 2026-11-18
signature: paused, VARIABLES has no `x` though the editor shows `int x = 1;` above the arrow
source: headmate (2026-11-18), same session as BB-BUG-072
cause: same as BB-BUG-072: `x` was typed after the build, so the running frame has no `x`; nothing on screen said so
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Debug, pause, type a new local above the arrow without restarting.

Expected: VARIABLES and the status say the file changed since the debug build.
Actual: VARIABLES lists the old locals with no notice.
