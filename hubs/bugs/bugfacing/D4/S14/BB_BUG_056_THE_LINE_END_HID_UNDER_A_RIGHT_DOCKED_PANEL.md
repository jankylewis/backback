# BB-BUG-056 — The line end hid under a right-docked panel

id: BB-BUG-056
family: UIX
status: FIXED
date: 2026-11-06
signature: scrolled fully right, a long line still ends under the Explorer; the bar's thumb is at its end
source: headmate report 2026-11-06 (screenshot: Explorer docked right, line 3 cut at "involve")
cause: the text width was the window less the left edge only; a panel docked right was not left out
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Settings: tool window on the right. Open a file with a line wider than the
editor, press End (or scroll right as far as it goes).

Expected: the line's last characters show left of the Explorer.
Actual: the scroll stops one panel width short; the end stays under the
Explorer. The terminal had the same fault: 244 columns instead of 204, so
its lines ran under the panel too.
