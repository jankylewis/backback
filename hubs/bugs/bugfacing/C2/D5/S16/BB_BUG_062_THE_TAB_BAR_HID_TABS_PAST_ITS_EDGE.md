# BB-BUG-062 — The tab bar hid tabs past its edge

id: BB-BUG-062
family: UIX
status: FIXED
date: 2026-11-10
signature: the last tab's name is cut at the Explorer edge (`C# OpenStandalon`); no scroll reaches it or the tabs after it
source: headmate report 2026-11-10 (screenshot: MbTx, three tabs, the third cut)
cause: the bar was a fixed-height flex row with `overflow_hidden()` (`bar/bb_view.rs`): tabs past the right edge were clipped, nothing scrolled the row, and a newly opened tab could be active yet out of sight
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Open more files than the tab bar can show, in one editor group.

Expected: as in VS Code, the bar scrolls sideways (wheel or trackpad) and
the active tab is always in sight.
Actual: the tabs past the edge are cut off and cannot be reached; the
active tab can be hidden.
