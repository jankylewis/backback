# BB-BUG-063 — Dragging the pane divider selected the text under it

id: BB-BUG-063
family: UIX
status: FIXED
date: 2026-11-11
signature: dragging the line between two editor panes highlights the left pane's text and scrolls it
source: headmate report 2026-11-11 (screen video: MbTx, a runsettings file left, a spec right)
cause: the editor line's mouse-move handler extended the selection on every move with the button held (`event.dragging()`), whatever the press began on: the divider, a scroll bar thumb, a tab
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Split the editor in two, click in the left pane's text (its cursor on a
line), then drag the line between the panes left and right.

Expected: only the panes' widths change.
Actual: the left pane's text under the pointer is selected, growing and
scrolling with the drag. The next key typed would replace that text.
