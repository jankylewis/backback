# BB-UIX-001 — The cursor is drawn as a text character inside the line

id: BB-UIX-001
family: UIX
status: OBSERVED
date: 2026-11-01
signature: push_run(&mut runs, '│'
source: `bb_crates/shell/desktop/src/editor/view/bb_line.rs:51` (cursor inside the line)
source: `bb_crates/shell/desktop/src/editor/view/bb_line.rs:68` (cursor at line end)
links: fix=no lesson=no evidence=no toolgap=none

## What it is

While building a line's text runs, the view inserts a `'│'` character at the
cursor column. The glyph takes a full Menlo cell, so every later character is
laid out one column to the right.

## How to confirm

Read `bb_line.rs:50-69`: the cursor is a pushed char, not an overlay. On screen,
the line is one column wider than the file text.

## Fix direction (not proven)

Draw the cursor as a 1-2 px quad positioned at column x (monospace advance x
column, or the shaped line's x for the index), painted over the text; never part
of the text. Red-first test: the shaped text of a line is byte-equal to the file
line wherever the cursor is.
