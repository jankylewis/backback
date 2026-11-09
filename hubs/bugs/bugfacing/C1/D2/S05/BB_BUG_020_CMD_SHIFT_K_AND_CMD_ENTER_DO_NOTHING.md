# BB-BUG-020 — Cmd+Shift+K and Cmd+Enter do nothing

id: BB-BUG-020
family: UIX
status: FIXED
date: 2026-11-03
signature: Cmd+Shift+K does nothing; Cmd+Enter then a key types into the line
source: sweep 4 on the installed app at `38baa8b`, `docs/plan/readback/V3/SWEEP04/bb_READBACK.md`
cause: BB-UIX-010
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Put the cursor on a line, press one of these keys of VS Code's macOS
keymap, Cmd+S.

| Key | Expected (VS Code) | Actual |
|---|---|---|
| Cmd+Shift+K | the line goes, with its line break | nothing |
| Cmd+Enter / Cmd+Shift+Enter | a new line below / above, with the indent | nothing; the next key types into the line |

Goal: BBEOD02 (editor).
