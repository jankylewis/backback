# BB-BUG-059 — Deleted folders took the recent list's places

id: BB-BUG-059
family: UIX
status: FIXED
date: 2026-11-06
signature: the start page lists 2-3 recent workspaces though more were opened
source: self-detected 2026-11-06 while building the 10-entry list (PR #91)
cause: the saved list kept folders gone from disk; 5 of its 8 places were deleted test temp folders, hidden by the page
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Open workspaces from temp folders that are later deleted (test and
readback runs do), then open real ones.

Expected: the start page shows the recent workspaces, up to the limit.
Actual: the page hides the gone folders, but they still use the places,
so only 3 of 8 showed.
