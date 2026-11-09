# BB-BUG-041 — The save prompt acts on the wrong tab after tabs move

id: BB-BUG-041
family: UIX
status: FIXED
date: 2026-11-04
signature: `bb_tabs_state ... asking=true` while the asked tab moved; Discard closed another file
source: split-editor readback on the branch build (step 6 of 6), before the PR
cause: the save prompt held the tab's index, not its file
links: fix=yes lesson=no evidence=yes toolgap=none

## Steps

Edit a.cs, press Cmd+W (the prompt asks about a.cs). While it asks, pin
another tab from its menu, or move a tab to the other group (Cmd+\).
Press Discard.

Expected: a.cs closes, its edits dropped, as asked.
Actual: the tab now at a.cs's old place closed (or was saved by Save);
a.cs stayed open. On master since #73 (pins); the split made it easy.
