# BB-PRF-009 — A first open of a many-folder tree still waits for the walk

id: BB-PRF-009
family: PRF
status: FIXED
date: 2026-11-20
signature: first_frame_ms ~220 on the first open of a folder with thousands of sub-folders (no saved choice)
source: `docs/plan/bench/bb_GENERAL/RERUNS/BBEOD37_BINARY_SIZE_AND_DNAAX.md` (section 2)
source: `bb_core/lang/src/detect.rs` `detect_until` (one-thread `ignore` walk)
source: `shell/desktop/src/app/bb_launch/open/langs.rs` `resolve` at `243c495` (joined the early walk before the first frame)
observed: dna-ax tree, BBEOD37: base 184 ms, fix 350 ms exec to first frame; saved choice 184 vs 184
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

BB-PRF-007 starts the walk right after the arguments and joins it in
`build_window`. The app reaches its first frame ~56 ms after start; a
walk longer than that still holds the window for the rest. The dna-ax
tree's walk is 229 ms: 13,178 entries in 4,873 folders, ~47 µs a folder
(open, read, ignore files), so folders, not files, set its cost. A
saved choice ends the walk at the first file of each ticked language,
so only the first open of a folder (or one never walked to its end)
pays it.

## How to confirm it

Launch the base and the fix bundle on the tree with an empty state
folder, then with a saved choice; time `bb_lang::detect` on it with a
small release program and compare with the growth of `first_frame_ms`.

## Ruling (matemaster, 2026-11-20, BBEOD38 brief)

"The first frame must NOT wait for the walk. Show the window at once
(Explorer, editor, tabs work); the language set, status bar languages,
menus and server starts follow when detect finishes (event to the
window). Keep everything a saved choice gives today. A parallel walk
alone is not enough (68 ms may still exceed the ~56 ms start)."

## Fixed

BBEOD38: the window opens with no language active and takes the
languages when the walk ends. Fix file: same path under `failfix/`.
