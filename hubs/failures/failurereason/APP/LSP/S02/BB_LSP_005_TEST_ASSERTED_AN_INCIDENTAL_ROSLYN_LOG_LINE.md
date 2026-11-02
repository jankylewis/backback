# BB-LSP-005 — A test asserted a Roslyn log line that only a side effect writes

id: BB-LSP-005
family: LSP
status: FIXED
date: 2026-11-03
signature: no server line of the pull after the load
source: CI run 41501823925 (PR #30, an icons-only change), `edge/intel/tests/server_log.rs:51`
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

`server_log_lines_after_the_load_reach_the_log` required a log line tagged
`[textDocument/diagnostic]` after the load. Roslyn writes no such line for
the pull itself: locally the tagged lines are side effects logged in the
request's scope (the MSBuild BuildHost starting lazily, the file-based
programs project system reloading, the latter even after "Shutting down").
On the CI runner the BuildHost was already up, so no line carried the tag.

## How it shows

Red only on CI, once, on a PR that touched no Rust code; 3 of 3 local runs
green. The log after the load held `[config]` lines and `[info] Shutting
down`, so log forwarding after the load did work.
