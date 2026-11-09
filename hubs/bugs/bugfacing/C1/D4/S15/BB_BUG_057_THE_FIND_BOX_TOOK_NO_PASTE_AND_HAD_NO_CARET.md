# BB-BUG-057 — The find box took no paste and had no caret

id: BB-BUG-057
family: UIX
status: FIXED
date: 2026-11-06
signature: Cmd+V in Find in Files does nothing; no caret shows; Left/Right do nothing
source: headmate report 2026-11-06 (screenshot: empty Find in Files box)
cause: the box kept a plain string; keys with Cmd were dropped and text only went on the end
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Copy a word. Press Cmd+Shift+F, press Cmd+V, then Left.

Expected: the word goes in; a caret shows and moves, as in VS Code.
Actual: nothing is pasted; there is no caret, and Left/Right do nothing.
The same box serves Go to File, Go to Symbol and the Explorer's name box,
so all of them had the fault.
