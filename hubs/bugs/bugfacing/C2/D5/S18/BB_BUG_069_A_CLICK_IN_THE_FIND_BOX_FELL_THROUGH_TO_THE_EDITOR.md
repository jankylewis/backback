# BB-BUG-069 — A click in the Find box fell through to the editor

id: BB-BUG-069
family: UIX
status: FIXED
date: 2026-11-17
signature: after a click on a Find in Files button (Project, Aa) that lies over the code, Alt+C / Alt+W / Alt+X and typing go to the editor, not the box
source: self-detected: the gate's quick-open window test went red when the new title bar (BB-FEAT-090) moved the editor 32 px down, under the box's buttons
cause: the box and the menus were drawn above the editor but did not occlude it; a press reached the editor's own mouse-down too, which took the focus (BB-BUG-060) or moved the caret
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Cmd+Shift+F, click Project in the box where it lies over code lines, press Alt+C.

Expected: Match Case turns on.
Actual: nothing in the box changes; the editor has the focus.
