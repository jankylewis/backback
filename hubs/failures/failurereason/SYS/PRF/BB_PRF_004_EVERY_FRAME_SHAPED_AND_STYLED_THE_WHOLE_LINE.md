# BB-PRF-004 — Every frame shaped and styled the whole line

id: BB-PRF-004
family: PRF
status: FIXED
date: 2026-11-04
signature: UI work grows with a line's length, seconds per key past 100 000 chars
source: `bb_crates/shell/desktop/src/editor/view/hscroll.rs` (`before_rows`), `bb_crates/shell/desktop/src/editor/view/mod.rs` (`caret_x`), `bb_crates/shell/desktop/src/editor/view/bb_line/bb_runs.rs` (`styled_line`)
observed: BB-BUG-032
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

Per frame, the widest visible line was shaped to size the scroll, the
cursor line shaped again for the caret, and the row's runs built char by
char with a scan of every highlight span for each char: shaping is
linear in the line, the run build is chars x spans (229 000 x about
40 000 on a minified line). Only about 200 chars fit on screen.

## How to confirm it

Time `goto:<line>:<col> right state` on a one-line file of 100 000+
chars against the same on a short file.
