# BB-PRC-009 — A child group outlives an owner that skips its cleanup

id: BB-PRC-009
family: PRC
status: FIXED
date: 2026-11-20
signature: a `dotnet`, `MSBuild.dll`, `VBCSCompiler` or build step with ppid 1 and no backback running
source: `bb_crates/bb_core/process/src/bb_group.rs` at 5ad7d19 (drop, quit hook and signal handler end a group)
source: `bb_crates/edge/dap/src/target.rs` at 5ad7d19 (the debugger's `dotnet build` in no owned group)
observed: matemaster 2026-11-20: 5 Roslyn `BuildHost` + MSBuild node pairs left 16.5 h (BB-BUG-094)
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

Every owned group was ended by code backback runs: `OwnedGroup` drop, the
quit hook, the SIGTERM/HUP/INT handler. A SIGKILL, an abort (release
builds abort on panic), `std::process::exit` (the test watchdog) or
nextest killing a timed-out test runs none of it. What then survives is
what does not watch backback: a `dotnet build` and its `Exec` step,
`VBCSCompiler` from the debugger's build (it had no group at all).

Servers on stdin (Roslyn 5.13, rust-analyzer, pyright) did end by
themselves on stdin end of file within 5 s in every probe on 2026-11-20,
also with Roslyn's `BuildHost` and two MSBuild nodes live, and after a
SIGKILL to Roslyn alone. Why the five pairs of 2026-11-19 lived 16.5 h is
not proven (their pids were gone before BBEOD42).

## How to confirm

`shell/desktop/tests/no_orphan_servers.rs`: a real `backback` killed while
it builds, debugs, panics or holds the unsaved-tabs prompt; `ps` its tree.
