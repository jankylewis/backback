# BB-UIX-007 — Tab and the Cmd deletes had no line-wise commands

id: BB-UIX-007
family: UIX
status: FIXED
date: 2026-11-03
signature: Tab with a selection replaces it
source: `bb_crates/shell/desktop/src/editor/input/bb_keys.rs` (`text_key`'s default arm, `command_key`); bugs BB-BUG-014, BB-BUG-015
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

`text_key` sent every key without an arm of its own to `insert(key_char)`,
which replaces a selection with the typed char: right for a letter, wrong
for Tab, whose char is a tab. Shift+Tab reached the same arm. `command_key`
had no arm for Backspace or Delete with Cmd, so they returned nothing. The
editor had no line-wise command at all: nothing to indent, outdent, or
delete to a line's edge.

## How it shows

A selected line vanishes on Tab and a tab character takes its place
(`\t    void M() { }` for a one-line selection); Shift+Tab does the same;
Cmd+Backspace leaves the file unchanged.

## How to confirm it

Select a line, press Tab, save: the bytes lose the line. Before the fix
the key map had no `"tab"` arm.
