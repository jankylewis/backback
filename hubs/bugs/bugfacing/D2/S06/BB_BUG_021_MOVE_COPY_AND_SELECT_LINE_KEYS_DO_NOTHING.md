# BB-BUG-021 — The move, copy and select-line keys do nothing

id: BB-BUG-021
family: UIX
status: FIXED
date: 2026-11-03
signature: Alt+Down does nothing; Cmd+L then a key types at the cursor
source: sweep 4 on the installed app at `38baa8b`, `docs/plan/readback/V3/SWEEP04/bb_READBACK.md`
cause: BB-UIX-010
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Put the cursor on a line, press one of these keys, Cmd+S.

| Key | Expected (VS Code) | Actual |
|---|---|---|
| Alt+Up / Alt+Down | the line swaps with the one above / below | nothing |
| Shift+Alt+Up / Shift+Alt+Down | a copy of the line above / below | nothing |
| Cmd+L | the whole line selected, with its break | nothing; the next key types at the cursor. Fixed: `select_line` (#60) |
| Cmd+] / Cmd+[ | the line moves in / out one level | Forward / Back: the app binds them first (Xcode style, `open.rs`) |

## Status

FIXED on branch `bb-bug-021-move-copy-lines`: Alt+Up/Down move lines,
Shift+Alt+Up/Down copy them (`move_lines`, `copy_lines`); Cmd+L since #60.
Cmd+] / Cmd+[ stay Back / Forward until the headmate rules (line indent is
on Tab / Shift+Tab with a selection).

Goal: BBEOD02 (editor).
