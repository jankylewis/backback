# BB-GAT-014 — A real server thread woke gpui's test scheduler

id: BB-GAT-014
family: GAT
status: FIXED
date: 2026-11-19
signature: `Detected activity on thread Some("bb-pyright") ... Your test is not deterministic.`
source: `bb_crates/shell/desktop/src/app/bb_launch/open/lang_tests/tools/live/bb_kit.rs` (`Live::attach`, `pump`)
observed: first real-server window tests: 3 of 3 runs panicked in gpui's `test_scheduler.rs`
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

A test build starts no server (`connect_rust`/`connect_python` return
under `cfg!(test)`). Attaching the real worker's channel to the window
made its thread wake a gpui task; the test scheduler refuses foreign
wakes. Earlier runs got further only by timing.

## How to confirm

Attach `start_python`'s receiver straight to `attach_server` in a test.
