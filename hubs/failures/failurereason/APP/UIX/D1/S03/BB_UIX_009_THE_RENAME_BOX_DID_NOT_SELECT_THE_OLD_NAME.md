# BB-UIX-009 — The rename box did not select the old name

id: BB-UIX-009
family: UIX
status: FIXED
date: 2026-11-03
signature: HalfHalve
source: `bb_crates/shell/desktop/src/intel/glue/asks.rs` (`rename_start`), `intel/glue/bb_keys.rs` (`rename_box`); bug BB-BUG-018
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

`rename_start` filled the box with the symbol's name and `rename_box`
appended every typed char to it. VS Code and Rider open the box with the
name selected, so the first key replaces it; a user who types the new name
straight away, as every user does, got the old name with the new one glued
on.

## How it shows

F2 on `Half`, `Halve` typed, Enter, Enter: `HalfHalve` in every file.

## How to confirm it

`RenameInput` had no notion of a selection; the window rig pressed
Backspace once before typing, which hid it for the one-char symbol.
