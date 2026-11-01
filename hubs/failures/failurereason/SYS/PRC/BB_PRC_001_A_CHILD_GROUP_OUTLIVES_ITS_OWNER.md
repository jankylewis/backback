# BB-PRC-001 — A child process group outlives its owner

id: BB-PRC-001
family: PRC
status: FIXED
date: 2026-11-01
signature: process left after quit
source: `bb_crates/bb_core/process/src/bb_group.rs` (the shared registry)
source: `docs/plan/readback/bb_FOUNDATION/BBEOD08/REAP.md` (red witnesses)
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

Killing only a leader process left its descendants: a test runner's host,
Roslyn's real server under its launcher, a debuggee under netcoredbg. They
kept running after a cancel, a drop or the app's quit.

## How to confirm

Record the app's process tree, end the app, and look for any recorded pid.
