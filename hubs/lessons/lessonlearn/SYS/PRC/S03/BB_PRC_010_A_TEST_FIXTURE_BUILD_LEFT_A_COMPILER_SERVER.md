# BB-PRC-010 — lesson

id: BB-PRC-010
family: PRC
status: FIXED
source: BBEOD43 (matemaster brief 2026-11-21, from BBEOD42 Open)

## Who believed what

The product's builds pass `-nodeReuse:false -p:UseSharedCompilation=false`,
so no build of ours leaves a compiler server.

## What changed the belief

The test fixtures built with their own plain `dotnet build`. And a process
count cannot prove it either way: a server someone else started serves the
build, so no new process appears; only the compiler's own log tells.

## Rule left behind

Start a tool family from one helper and let a guard refuse any other start.
Prove "used no server" from the tool's own record, not from the process list.
