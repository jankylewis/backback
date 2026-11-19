# BB-UIX-013 — The stop was far from the run

id: BB-UIX-013
family: UIX
status: FIXED
date: 2026-11-03
signature: a user cannot find how to stop what they started
source: `bb_crates/shell/desktop/src/bb_explorer/panels/tests_panel/view/mod.rs` (`toolbar`)
observed: the headmate's report (BB-BUG-026)
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

A spec was started with the ▶ on its own row, but the only stop was a
`Stop` link in the toolbar, two rows above the filter, in the link colour
of `Run failed` and `Category`, shown whether or not anything ran. The
stop worked (it killed the process group); nobody saw it.

## How it shows

A user waits for a long test to end because they see no way to stop it.

## How to confirm it

Start a run from a row; is a stop on that same row?
