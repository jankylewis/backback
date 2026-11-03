# BB-BUG-023 — The sidebar header does not show which side is chosen

id: BB-BUG-023
family: UIX
status: FIXED
date: 2026-11-03
signature: EXPLORER at the left, a Tests link at the far right
source: the headmate's feedback 2026-11-03
links: fix=yes lesson=no evidence=yes toolgap=none

## Steps

Open a folder and look at the sidebar's header; switch to the Tests side.

Expected: EXPLORER and TESTS side by side at the left; the chosen one in
pink, the other one in the text colour.
Actual: EXPLORER alone at the left and a "Tests" link at the far right; on
the Tests side, TESTS at the left and "Files" at the right; nothing showed
which side was chosen.

No failure mechanism: the header was drawn as a title and a link, by design.
