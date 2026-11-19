# BB-UIX-017 — Back needed an open tab and its arrow trusted a non-empty stack

id: BB-UIX-017
family: UIX
status: FIXED
date: 2026-11-19
signature: ← is live, a click (or Cmd+[) does nothing
source: `bb_crates/shell/desktop/src/editor/bb_tabs/model/history.rs` at bb82de2 (`go` returned when `Tabs::here` was `None`; `can_back` was `!back.is_empty()`)
observed: BB-BUG-080
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

Two rules for one control. The press needed the active tab's place (to
push it onto Forward) and returned when no tab was open; the arrow was
lit by "the stack is not empty", blind to whether a place could still be
reached. A place in a deleted file kept the arrow live and its press
failed in the open.

## How to confirm it

Jump inside the only tab, close it: ← is live, a press does nothing.
Delete the file of the last place: ← stays live.
