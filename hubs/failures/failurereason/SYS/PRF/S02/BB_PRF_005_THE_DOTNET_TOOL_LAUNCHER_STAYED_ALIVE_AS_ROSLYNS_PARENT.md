# BB-PRF-005 — The dotnet tool launcher stayed alive as Roslyn's parent

id: BB-PRF-005
family: PRF
status: FIXED
date: 2026-11-07
signature: two Roslyn processes per window: `~/.dotnet/tools/roslyn-language-server` (~49 MB) and its child `Microsoft.CodeAnalysis.LanguageServer`
source: self-measured (perf epic, 2026-11-07), process tree of the app on MbTx at idle
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

`roslyn-language-server` in the tools folder is the dotnet tool's
launcher: it starts the real server from the tool store and then waits
as its parent for the whole session. backback started the launcher, so
each window kept one extra process doing nothing.

## How it shows

`ps` under backback at idle: the launcher 49 MB RSS, 0% CPU; the server
517 MB. One launcher per window and per `bb open` / `bb probe`.
