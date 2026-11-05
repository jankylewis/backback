# BB-BUG-055 — The view did not follow the cursor

id: BB-BUG-055
family: UIX
status: FIXED
date: 2026-11-05
signature: arrow up from the top visible line moves the cursor off screen; the lines do not scroll
source: headmate request 2026-11-05 (screenshots: cursor on line 30 at the top, Up, line 29 stays hidden)
cause: no key path scrolled the line list; only Go to Definition, test jumps and script steps did
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Scroll so line 30 is the top line, put the cursor on it, press Up.

Expected (VS Code): the view scrolls one line so line 29 shows.
Actual: the cursor moves to line 29 out of view; the same at the bottom
with Down, Enter or Cmd+Down.
