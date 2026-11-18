# BB-PRC-008 — A debug-test run outlived its test binary

id: BB-PRC-008
family: PRC
status: FIXED
date: 2026-11-18
signature: `dotnet test <tmp>/Tests.csproj -c Debug --no-build --nologo --filter FullyQualifiedName=...` with ppid 1 at 0 % CPU after `cargo test` ended
source: `bb_explorer/panels/run/bb_debug/session/tests/mod.rs` (before: `start(..)` then `commands.send(Command::Stop)` and return)
observed: matemaster found the orphan (pid of `Sample.Spec.Waits`, 6+ min) holding a `cargo test | tail` pipe open
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

The debug session runs on a detached `bb-debug` thread that owns the
paused `dotnet test` run (`DebugHost`, an `OwnedGroup`). A session test
sent Stop and returned, or failed an assertion; the test binary then
exited and the OS ended that thread mid-teardown, without its drops, so
the run's group was never ended. The app is safe: quit runs
`bb_process::shutdown` (BB-PRC-001).

## How to confirm it

Drop the session right after a stop and `pgrep -f <temp project root>`.
