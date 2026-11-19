# BB-BUG-082 — A Debug start sent the breakpoints of a deleted file

id: BB-BUG-082
family: UIX
status: FIXED
date: 2026-11-19
signature: no status line; netcoredbg gets `setBreakpoints` for a path that is gone
source: BBEOD30 note; `session/control.rs:115` at 6f40103 (`breakpoints: view.debug.breakpoints.all()`)
cause: BB-UIX-019
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Set a breakpoint in B.cs (F9), delete B.cs outside the app, press F5
(or Debug on a test row).

Expected: B.cs's breakpoints are not sent; the status says
`Debug: breakpoints in B.cs dropped: the file no longer exists`.

Actual: every stored breakpoint was sent, B.cs's too, and nothing was
said; they stayed stored and were sent again at every start (code read
and the RED unit test; no live netcoredbg run in BBEOD31).
