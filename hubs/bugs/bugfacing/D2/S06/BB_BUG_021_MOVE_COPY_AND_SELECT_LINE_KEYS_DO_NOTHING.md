# BB-BUG-021 — The move, copy and select-line keys do nothing

id: BB-BUG-021
family: UIX
status: OPEN
date: 2026-11-03
signature: Alt+Down does nothing; Cmd+L then a key types at the cursor
source: sweep 4 on the installed app at `38baa8b`, `docs/plan/readback/V3/SWEEP04/bb_READBACK.md`
cause: BB-UIX-010
links: fix=no lesson=yes evidence=yes toolgap=none

## Steps

Put the cursor on a line, press one of these keys, Cmd+S.

| Key | Expected (VS Code) | Actual |
|---|---|---|
| Alt+Up / Alt+Down | the line swaps with the one above / below | nothing |
| Shift+Alt+Up / Shift+Alt+Down | a copy of the line above / below | nothing |
| Cmd+L | the whole line selected, with its break | nothing; the next key types at the cursor |
| Cmd+] / Cmd+[ | the line moves in / out one level | Forward / Back: the app binds them first (Xcode style, `open.rs`) |

## Not done yet

Moving and copying lines must keep CRLF, a last line without a final
newline, and a selection over several lines; each needs its own test.
Cmd+[ / Cmd+] need a ruling: Back/Forward (now) or indent/outdent
(VS Code); line indent is on Tab and Shift+Tab with a selection.

Goal: BBEOD02 (editor).
