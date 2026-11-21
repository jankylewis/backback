# BB-PRC-010 — A test fixture's build left a compiler server

id: BB-PRC-010
family: PRC
status: FIXED
date: 2026-11-21
signature: `VBCSCompiler -pipename:` with ppid 1 after a test run, in a group no backback registered
source: `bb_crates/shell/desktop/tests/bb_native_discovery.rs` and 9 more files at a9053fd (a plain `Command::new("dotnet")`)
observed: BBEOD42 gates 2026-11-20: a `VBCSCompiler` with ppid 1, group not backback's (fleet `evidence/BBEOD42_READY.md`, Open)
observed: 2026-11-21 23:21: one `VBCSCompiler` with ppid 1, group id not its own pid (no `setsid`), alive 40 min; not ours, not ended
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

A plain `dotnet build` hands each compile to the per-user compiler server
(`VBCSCompiler`), starting one when none runs. The server stays in the group
of the build that started it and lives on after the build, idle, for minutes.
Ten places in tests and two in the product started `dotnet` themselves; the
tests passed neither `-p:UseSharedCompilation=false` nor `-nodeReuse:false`.
When the test process ends, the server's parent is gone (ppid 1) and nothing
owns its group. Which test started the BBEOD42 server: not proven.

MSBuild worker nodes: a two-project build left none on 2026-11-21 (seen, one
run); `MSBUILDDISABLENODEREUSE=1` is set anyway.

## How to confirm

Run the build with `RoslynCommandLineLogFile=<file>`. `CompilerServer: server
- server processed compilation` = it used a server; `CompilerServer: tool -
using command line tool by design` = it compiled in its own process. A server
already running serves the build, so no new process shows: read the log.
`edge/dotnet/bb_sdk/tests/no_servers.rs` does both checks.
