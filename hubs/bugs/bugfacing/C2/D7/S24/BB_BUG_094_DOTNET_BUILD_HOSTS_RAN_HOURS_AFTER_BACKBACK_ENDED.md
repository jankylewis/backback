# BB-BUG-094 — dotnet build hosts ran hours after backback ended

id: BB-BUG-094
family: PRC
status: FIXED
date: 2026-11-20
signature: `dotnet --roll-forward LatestMajor …/BuildHost-netcore/…BuildHost.dll` and `MSBuild.dll /nodemode:1 /nodeReuse:false` with ppid 1, no backback running
source: matemaster report 2026-11-20 (BBEOD42 brief)
observed: 10 `dotnet` processes, ppid 1, 16.5 h, about 1.1 GB: 5 pairs started 22:00-22:15 on 2026-11-19 (the BBEOD31/32 readback and test window); ended by the matemaster, `ps` evidence gone
cause: BB-PRC-009
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Run backback (or its tests) on a C# solution; end it without its cleanup
(a SIGKILL, a crash, a test run killed) while a child is busy; `ps` for
`dotnet` with ppid 1.

Expected: every process backback started ends within a few seconds of it.

Actual: the pairs kept running for hours. The pair matches the command
shape of Roslyn's `BuildHost` and its MSBuild node (seen in a 2026-11-20
probe); why these ones did not end is not proven.
