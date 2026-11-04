# BB-BUG-039 — No pinned tabs and no bulk close

id: BB-BUG-039
family: UIX
status: FIXED
date: 2026-11-04
signature: a tab had only its × ; no right-click menu, no pin
source: headmate request 2026-11-04 ("pin the current file", "close all tabs to the right/left", "close all except pinned", "close all")
cause: not built (a VS Code feature the tab bar lacked)
links: fix=yes lesson=no evidence=yes toolgap=none

## Steps

Open several files; right-click a tab.

Expected (VS Code): a menu with Close, Close Others, Close to the Right,
Close to the Left, Close All Except Pinned, Close All, Pin / Unpin; a
pinned tab moves left, shows a pin, and survives every close but Close All.
Actual: nothing happened; tabs closed one × at a time.
