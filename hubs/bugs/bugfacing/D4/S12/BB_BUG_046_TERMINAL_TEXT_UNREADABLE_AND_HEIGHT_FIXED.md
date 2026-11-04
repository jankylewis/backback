# BB-BUG-046 — The terminal's text was unreadable and its height fixed

id: BB-BUG-046
family: UIX
status: FIXED
date: 2026-11-04
signature: near-black terminal text on the dark theme; no drag on the terminal's top line
source: headmate report 2026-11-04 (screenshot, build db96073: "the theme and fontsize of terminal is not readable"; "make this line draggable")
cause: the panel set a background but no text colour, so rows drew in gpui's default black; its height was a constant (240 px)
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Dark theme; press Ctrl+` and type `pwd`; try to drag the line above it.

Expected: text in the theme's text colour, the editor's font; the line drags.
Actual: dark grey-black text on the dark background; the line did not move.
