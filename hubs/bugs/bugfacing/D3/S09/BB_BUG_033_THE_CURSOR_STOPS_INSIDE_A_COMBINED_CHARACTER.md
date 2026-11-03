# BB-BUG-033 — The cursor stops inside a combined character

id: BB-BUG-033
family: UIX
status: FIXED
date: 2026-11-04
signature: Left from a line's end lands between `e` and its combining accent
source: night robustness sweep 2026-11-04 (`robust_sweep.py`, branch build), case "emoji + combining"
links: fix=yes lesson=no evidence=yes toolgap=none

## Steps

A line holding `e` + U+0301 (shown as one `é`) or an emoji sequence;
move the cursor over it with Left/Right, then select or delete.

Expected (VS Code): the cursor moves over a whole grapheme; a delete
takes the whole `é`.
Actual: the cursor moves by code point: six Lefts from the end of
`"a👍b é 中文";` stop at column 22, between `e` and the accent.

## Status

FIXED: Left, Right, Backspace and Delete step one grapheme. Word moves and
a click already stop at word or glyph edges; not re-checked by hand.
