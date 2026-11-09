# BB-BUG-038 — A second window wipes the first's Roslyn folder

id: BB-BUG-038
family: PRC
status: FIXED
date: 2026-11-04
signature: `progress.last() (2, 3) != (3, 3)`; failed: `.../XunitPlaywright/_XunitPlaywright.csproj`
source: gate runs 2026-11-04 02:15 and 03:40 (the server-thread test, under the full gate's parallel tests); BB-TG-017 named the project
cause: BB-PRC-005
links: fix=yes lesson=yes evidence=yes toolgap=BB-TG-017

## Steps

Open two workspace windows in one app (File > Open...), or run two intel
tests at once; close one.

Expected: each window's C# server keeps its own scratch build and log.
Actual: every worker used `backback-intel-<pid>`. Opening the second
removed the first's folder (Roslyn's design-time build output and log),
and closing either removed the other's: a project load could fail. Alone
the test passed 14 of 14; in the gate it failed twice.
