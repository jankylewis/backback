# BB-GAT-018 — A timed walk hold and a kept server link failed under CI load

id: BB-GAT-018
family: GAT
status: FIXED
date: 2026-11-20
signature: `the window waited for the walk` with `editor_open_ms=3093`; nextest `LEAK [ 12.004s] ... restart::a_roslyn_restart_reopens_only_the_csharp_tabs`
source: PR #127 CI run 44620984707 (macOS, nextest, 4 workers), log lines 563-583; `langs/bb_early.rs` `wait_while_held` and `first_frame/relay.rs` `relay_requests` at 8fe9ec6
observed: CI only; local runs (1 thread) were green
links: fix=yes lesson=yes evidence=no toolgap=none

## What it is

Two test-seam faults, no product wait. (1) The slow-walk hold let go after 3 s of wall clock; on CI the window took 3.09 s to open the 3,000-function file, so the held walk answered before the test looked. (2) The relay kept each real server `Link` in the log's `Arc` after the window let go, so no server shut down before the test process exited; Roslyn and its BuildHost outlived it and held nextest's pipe (LEAK). Orphaned BuildHost processes (ppid 1) from earlier local runs are the same leak.

## How to confirm it

Old seam: `a_held_walk_still_has_no_answer_after_the_old_three_second_cap` fails ("the walk answered while held"). Without the relay's `take()`: the restart test ends `1 of 4 server threads ended`.
