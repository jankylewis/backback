# BB-UIX-020 — A path that did not resolve was always called remote

id: BB-UIX-020
family: UIX
status: FIXED
date: 2026-11-19
signature: `Cannot open <opened root>/X: not on this machine`
source: `tests_panel/actions/navigation.rs` `jump_to` at 6f40103
observed: BB-BUG-083
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

`jump_to` maps a stack path to a local one and canonicalizes it; any
failure was reported as `not on this machine`, which is right for a CI
path and wrong for a file under the opened root that was deleted.

## How to confirm it

Go to a frame of a deleted file under the root and read the status.
