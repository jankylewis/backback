# BB-PRC-002 — `dotnet build` spawned from a background thread never finished

id: BB-PRC-002
family: PRC
status: FIXED
date: 2026-11-02
signature: Building Harness.slnx... (never ends)
source: `bb_crates/shell/desktop/src/bb_explorer/panels/build/run.rs`
source: `docs/plan/readback/FIXES/S02/BBEOD13/bb_READBACK.md`
observed: the same command finished in 2 s from a shell, from Python in its
  own process group, niced, and from the app's working folder; in the app it
  slept for 90 s with its two MSBuild nodes
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

The app spawned `dotnet build` on a gpui background-executor thread. On
macOS a process spawned from a background-QoS thread inherits the background
state (`ps` shows `N`); MSBuild's file I/O is throttled until the build looks
hung. Two smaller faults were fixed with it: stdin was the terminal for a
child in its own process group, and leftover MSBuild nodes could hold the
output pipe open.

## How to confirm

`ps -o stat` of a build the app started: `SN`, no progress for minutes.
