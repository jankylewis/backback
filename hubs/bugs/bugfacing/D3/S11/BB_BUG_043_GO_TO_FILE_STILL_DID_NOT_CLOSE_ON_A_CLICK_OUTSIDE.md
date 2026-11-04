# BB-BUG-043 — Go to File still did not close on a click outside

id: BB-BUG-043
family: BLD
status: FIXED
date: 2026-11-04
signature: headmate screenshot: Go to File over a split editor, still open after a click outside, after BB-BUG-042 was "fixed"
source: headmate report 2026-11-04 13:xx ("seems like u r failed to fix this? -> plz do rca")
cause: BB-BLD-003 (the running app was `be13e35`; the fix is `ef68ebf`, merged in `dd0ccf8`, never installed)
links: fix=yes lesson=yes evidence=yes toolgap=BB-TG-019

## Steps

Run the installed app, split the editor, press Cmd+Shift+O, click the
Explorer or either pane.

Expected: the box closes (BB-BUG-042).
Actual: it stayed: the installed build predates the fix. On a build with
the fix, a window test pressing an Explorer row, the focused pane's
text, the other pane and the status bar closes it each time.
