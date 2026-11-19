# BB-UIX-006 — Editor rows had no horizontal offset; the list clipped them

id: BB-UIX-006
family: UIX
status: FIXED
date: 2026-11-03
signature: cannot horizontally scroll
source: `bb_crates/shell/desktop/src/editor/view/mod.rs` (`uniform_list` rows), `view/bb_line/mod.rs` (`whitespace_nowrap` text); bug BB-BUG-013
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

The editor is a `uniform_list` of rows; each row laid its shaped line out
after the gutter with `whitespace_nowrap`, at the pane's width. The list
scrolls vertically only, and no state anywhere said how far a line was
scrolled sideways, so every row was drawn from its first char and clipped
at the pane's edge; the caret, clicks and popups all counted from a fixed
left edge. Every readback so far used fixtures that fit the pane, or real
files by line number, so no step ever had to see the end of a wide line.

## How it shows

A trackpad swipe does nothing; End hides the caret; a long line reads as
cut. The headmate saw it on a real test file.

## How to confirm it

Open a file with a line wider than the pane; swipe. Before the fix the
text never moves and `state` prints no `scroll_x`.
