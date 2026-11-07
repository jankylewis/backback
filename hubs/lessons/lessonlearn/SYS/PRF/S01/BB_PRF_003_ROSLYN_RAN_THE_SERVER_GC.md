# BB-PRF-003 — lesson

id: BB-PRF-003
family: PRF
status: FIXED
source: night soak 2026-11-04

## Who believed what

The leak probe watched the app's own RSS, threads and files: flat, so
"no leak".

## What changed the belief

Summing the children's RSS per round showed the language server growing
by 100 MB a round, 3.5x what the workstation GC keeps.

## Rule left behind

A resource audit counts every child process the app owns, not the app
alone; and a child's runtime defaults (GC mode, thread pools) are ours to
choose, since we start it.
