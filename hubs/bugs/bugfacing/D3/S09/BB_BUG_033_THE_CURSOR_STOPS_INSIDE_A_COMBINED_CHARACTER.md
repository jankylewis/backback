# BB-BUG-033 — The cursor stops inside a combined character

id: BB-BUG-033
family: UIX
status: OPEN
date: 2026-11-04
signature: Left from a line's end lands between `e` and its combining accent
source: night robustness sweep 2026-11-04 (`robust_sweep.py`, branch build), case "emoji + combining"
links: fix=no lesson=no evidence=yes toolgap=none

## Steps

A line holding `e` + U+0301 (shown as one `é`) or an emoji sequence;
move the cursor over it with Left/Right, then select or delete.

Expected (VS Code): the cursor moves over a whole grapheme; a delete
takes the whole `é`.
Actual: the cursor moves by code point: six Lefts from the end of
`"a👍b é 中文";` stop at column 22, between `e` and the accent.

## Not proven

Which keys besides Left/Right split a grapheme (Backspace, Delete, word
moves, a click); each needs its own test. `unicode-segmentation` is in
`Cargo.lock` already (a transitive dependency).
