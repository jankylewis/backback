# BB-BUG-072 — The hover showed a type for code that was not running

id: BB-BUG-072
family: DAP
status: FIXED
date: 2026-11-18
signature: paused, the mouse on `x` shows `(local variable) int x`, no value
source: headmate (2026-11-18): the tab showed ● (unsaved); `int x = 1;` was typed after the debug build
cause: the running code is the session's build, which has no `x`; the debugger could not evaluate it, and the hover fell back to Roslyn's type of the edited text without saying why
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Debug, pause, type `int x = 1;` in the paused file, rest the mouse on `x`.

Expected: The hover says the file changed since the debug build.
Actual: It shows Roslyn's `(local variable) int x`.
