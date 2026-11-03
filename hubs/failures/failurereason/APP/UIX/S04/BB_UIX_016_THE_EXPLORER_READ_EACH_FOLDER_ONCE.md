# BB-UIX-016 — The Explorer read each folder once

id: BB-UIX-016
family: UIX
status: FIXED
date: 2026-11-04
signature: tree rows differ from `ls -A` after an outside change
source: `bb_crates/bb_core/explorer/src/tree/bb_model.rs` (`toggle` reads a folder only while `children` is `None`)
observed: BB-BUG-037
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

A folder's children were read on its first expand and kept for the
life of the window; nothing re-read them.

## How to confirm it

Expand a folder, change it from outside, compare the rows with `ls -A`.
