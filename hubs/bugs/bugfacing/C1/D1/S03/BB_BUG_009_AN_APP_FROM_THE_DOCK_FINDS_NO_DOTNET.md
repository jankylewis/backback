# BB-BUG-009 — an app opened from the Dock finds no `dotnet`

id: BB-BUG-009
family: SHIP
status: FIXED
date: 2026-11-03
signature: 1 project failed to load / debug I/O: No such file or directory
source: readback of `/Applications/backback.app` at `55fac1d`, launched with launchd's environment (`env -i`, PATH `/usr/bin:/bin:/usr/sbin:/sbin`)
cause: BB-ENV-001
links: fix=yes lesson=yes evidence=yes toolgap=BB-TG-012

## Steps

Open backback from the Dock or Finder (not from a terminal); open a C#
solution; wait for the C# server; press F5 or Cmd+Shift+B.

Expected: projects load, Build and F5 run `dotnet`.
Actual: "Ready — 1 project failed to load: Debuggee.csproj"; F5 says "Debug
failed: debug I/O: No such file or directory". launchd gives the app
`/usr/bin:/bin:/usr/sbin:/sbin`; `dotnet` is in `/usr/local/share/dotnet`.
Every earlier readback started the app from a terminal and inherited its
`PATH` (BB-TG-012), so none saw it.

Goal: BBEOD07 (macOS release).
