# BB-UIX-012 — A fold marker with no fold

id: BB-UIX-012
family: UIX
status: FIXED
date: 2026-11-03
signature: a row shows ▾ but a click on it does nothing
source: `bb_crates/shell/desktop/src/bb_explorer/panels/tests_panel/view/rows/mod.rs` (`spec_rows`)
observed: the headmate's report (BB-BUG-024)
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

The namespace and class heads were drawn as text that began with a fixed
`▾`: a label, with no state and no click handler. Only the project row had
an `expanded` flag. The marker promised a fold the row could not do.

## How it shows

A click on a namespace or class head does nothing; the marker never turns.

## How to confirm it

Look for a ▾ / ▸ written as a literal in a label instead of from state.
