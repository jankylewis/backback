# BB-BUG-022 — A click right of a line's end does nothing

id: BB-BUG-022
family: UIX
status: FIXED
date: 2026-11-03
signature: a click past the text leaves the cursor where it was
source: the headmate's report 2026-11-03 (a click right of line 21 of a metadata file left the cursor on line 20)
cause: BB-UIX-011
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Click in the empty space right of a line's last char.

Expected (VS Code): the cursor goes to the line's end; on an empty line, to
its start.
Actual: nothing; the cursor stays where it was. The current line's band also
ended right after its text instead of spanning the pane.

Goal: BBEOD02 (editor).
