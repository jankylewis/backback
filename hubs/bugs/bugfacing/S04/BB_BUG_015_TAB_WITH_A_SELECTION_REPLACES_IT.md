# BB-BUG-015 — Tab with a selection replaces it; Shift+Tab inserts a tab

id: BB-BUG-015
family: UIX
status: FIXED
date: 2026-11-03
signature: Tab with a selection replaces it
source: step-8 sweep (cases E1, E2) on the installed app at `0b6406d`, `docs/plan/readback/V2/SWEEP01/bb_READBACK.md`
cause: BB-UIX-007
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Select a whole line (Shift+Down from its start); press Tab; Cmd+S. Then the
same with Shift+Tab.

Expected: Tab indents the selected lines one level and Shift+Tab outdents
them, as VS Code and Rider do; the text stays. In a C# file the indent is
four spaces.
Actual: the selected line is deleted and a tab character takes its place
(`\t    void M() { }`); Shift+Tab does the same. Text is lost on a key
every editor user presses.

Goal: BBEOD02 (editor). Critical: text loss.
