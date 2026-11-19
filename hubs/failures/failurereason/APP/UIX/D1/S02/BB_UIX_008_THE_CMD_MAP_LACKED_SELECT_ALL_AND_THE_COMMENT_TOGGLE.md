# BB-UIX-008 — The Cmd map lacked Select All and the comment toggle

id: BB-UIX-008
family: UIX
status: FIXED
date: 2026-11-03
signature: Cmd+A does nothing
source: `bb_crates/shell/desktop/src/editor/input/bb_keys.rs` (`command_key`); bugs BB-BUG-016, BB-BUG-017
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

`command_key` listed the Cmd keys the editor had been given one by one
(save, close, undo, redo, the arrows, tab numbers) and answered `None` to
every other, so Cmd+A and Cmd+/ did nothing at all: no Select All command
and no line-comment command existed in the text model. The Cmd table of
VS Code and Rider had never been swept as a whole; keys were added as
they were missed.

## How it shows

Cmd+A then a typed char: the char goes in at the cursor, nothing was
selected. Cmd+/: the file does not change.

## How to confirm it

`command_key` has no `"a"` or `"/"` arm.
