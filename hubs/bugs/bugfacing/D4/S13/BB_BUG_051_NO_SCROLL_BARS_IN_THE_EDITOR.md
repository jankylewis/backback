# BB-BUG-051 — The editor had no scroll bars

id: BB-BUG-051
family: UIX
status: FIXED
date: 2026-11-05
signature: a tall or wide file showed no sign of more content and nothing to drag
source: headmate request 2026-11-05 ("can we show the horizontal scroll bar and vertical scroll bar too if the file needs to scroll to view oversighted content?")
cause: not built: lines scrolled by wheel, trackpad and keys only
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Open a file taller than the pane, or with a line wider than it.

Expected: a bar on the right (and at the bottom for a wide line), as VS Code; its thumb drags.
Actual: no bar.
