# BB-GAT-020 — A kill test acted on a timer before its processes existed

id: BB-GAT-020
family: GAT
status: FIXED
date: 2026-11-20
signature: servers missing ["sleep 600"], mark bb_mark building printed true
source: PR #130 CI run 44768652825 (768/769), `no_orphan_servers` `a_backback_that_panics_while_it_builds_leaves_no_process_behind` at `orphans/run.rs:60`
links: fix=yes lesson=no evidence=yes toolgap=none

## What it is

The script ended backback by itself on a timer (`mark:building sleep:8000
crash:panic`; the quit test: `BB_KEYS_QUIT` 1.5 s after `mark:up`), while the
test waited for processes (`sleep 600`) to appear. On the slow CI runner the
build had not reached its step after 8 s: backback panicked first, the
precondition was never met, and the wait ran its full 240 s on a dead app.
Family: every scenario that acts on a mark or a sleep instead of on the
observed process tree.

## How to confirm

Slow the build (`hang::fixture(_, true)`: `sleep 12` before the step): the
old keys fail the same way locally.
