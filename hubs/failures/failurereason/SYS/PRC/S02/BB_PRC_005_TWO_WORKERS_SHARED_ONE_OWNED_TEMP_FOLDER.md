# BB-PRC-005 — Two workers shared one owned temp folder

id: BB-PRC-005
family: PRC
status: FIXED
date: 2026-11-04
signature: a Roslyn project load fails only when two workers run in one process
source: `bb_crates/shell/desktop/src/intel/bb_worker/mod.rs` (`prepare`, before the fix: `OwnedTempDir::create("backback-intel")`)
observed: BB-BUG-038
links: fix=yes lesson=yes evidence=yes toolgap=BB-TG-017

## What it is

`OwnedTempDir::create` (BB-PRC-004, #67) names a folder by the pid only
and removes a folder of that name first ("this process's own earlier
one"); its drop removes it too. The intel worker runs once per window,
and tests run two at once, so workers removed each other's folders.

## How to confirm it

Two `intel_folder()` calls in one process: the same path before the fix.
