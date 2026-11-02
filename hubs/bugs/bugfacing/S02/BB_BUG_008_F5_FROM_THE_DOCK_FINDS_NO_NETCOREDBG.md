# BB-BUG-008 — F5 in the installed app finds no netcoredbg

id: BB-BUG-008
family: SHIP
status: FIXED
date: 2026-11-03
signature: Debug failed: netcoredbg not found
source: readback of `/Applications/backback.app` at `55fac1d`, opened with `open`
links: fix=yes lesson=no evidence=yes toolgap=none

## Steps

Install from master; open the app as a user does (Dock, Finder or `open`);
open a console project; F9 on a line; F5.

Expected: the build runs and the session pauses on the breakpoint.
Actual: "Debug failed: netcoredbg not found; set BACKBACK_NETCOREDBG or
install netcoredbg (MIT) and put it on PATH". An app opened from the Dock
gets neither the variable nor the shell's `PATH`; netcoredbg sits in
`~/.local/share/backback/netcoredbg`. The debugger readbacks before this set
`BACKBACK_NETCOREDBG` themselves, so they never saw it.

Goal: BBEOD17 (debugger in the app).
