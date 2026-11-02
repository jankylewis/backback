# BB-BUG-012 — The macOS Ctrl text keys do nothing

id: BB-BUG-012
family: UIX
status: FIXED
date: 2026-11-03
signature: Ctrl+A / Ctrl+E leave the cursor where it was
source: BB-BUG-011 modifier sweep in `/Applications/backback.app` at `24aa57c`
links: fix=yes lesson=no evidence=yes toolgap=none

## Steps

Put the cursor inside a line; press Ctrl+E, then Ctrl+A.

Expected (every Cocoa text view; VS Code on macOS): Ctrl+A/E go to the
line start/end, Ctrl+B/F one char left/right, Ctrl+P/N one line up/down,
Ctrl+H/D delete back/forward.
Actual: nothing moved (cursor stayed at 1:4 and 1:2). No stray letter
was typed.

Goal: BBEOD02 (editor).
