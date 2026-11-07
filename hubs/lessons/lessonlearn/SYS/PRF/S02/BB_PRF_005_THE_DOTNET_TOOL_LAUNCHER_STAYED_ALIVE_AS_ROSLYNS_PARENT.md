# BB-PRF-005 — lesson

id: BB-PRF-005
family: PRF
status: FIXED
source: perf epic, 2026-11-07

## Who believed what

The process list was "backback + Roslyn"; nobody had looked at what
`roslyn-language-server` itself is.

## What changed the belief

A per-process snapshot of the app's tree at idle, not a total.

## Rule left behind

Measure resources per process, by name, before tuning: a total hides a
process that should not exist. Idle RSS on macOS swings with page
compression (259-554 MB for one setting); compare settings only over
several runs, and ship only what beats that spread.
